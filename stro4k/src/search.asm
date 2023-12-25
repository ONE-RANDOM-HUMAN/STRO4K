NO_EVAL equ 80000000h
BOUND_NONE equ 00b
BOUND_LOWER equ 01b
BOUND_UPPER equ 10b
BOUND_EXACT equ 11b

F_PRUNE_MARGIN equ 78
STATIC_NULL_MOVE_MARGIN equ 63
DELTA_BASE equ 178
DELTA_IMPROVING_BONUS equ 11

section .rodata
DELTA_PRUNE_PIECE_VALUES:
    dw 114
    dw 425
    dw 425
    dw 648
    dw 1246

default rel
section .text

%ifdef EXPORT_SYSV
    extern search_print_info_sysv
    global alpha_beta
    global root_search_sysv

root_search_sysv:
    push r15
    push r14
    push r13
    push r12
    push rbx
    push rbp

    mov rbx, rdi
    mov r12d, esi
    mov r11d, edx
    call root_search
    mov eax, ebx

    pop rbp
    pop rbx
    pop r12
    pop r13
    pop r14
    pop r15
    ret

%else
thread_search:
    push rsp
    pop rbx

%ifdef EXPORT_SYSV
    xor r12d, r12d ; temp
    mov r11d, -1
%endif
    call root_search

    lock dec byte [RUNNING_WORKER_THREADS]
    jmp _start.exit
%endif
; search - rbx
; time should be calculated before calling root_search
; returns best move in ebx
root_search:
    mov qword [rbx + Search.nodes], 0

    ; r13d - depth
    ; r14d - last score
    ; r15d - best move - does not need to be initialised
    ; mov r13d, 1
    xor r13d, r13d
    xor r14d, r14d
.iterative_deepening_head:
    inc r13d
    ; ebp - window
    ; esi - alpha
    ; edi - beta
    mov ebp, 24

    mov esi, r14d
    lea edi, [rsi + rbp]
    sub esi, ebp
.do_search:
    ; clamp alpha and beta
    mov edx, MIN_EVAL
    cmp esi, edx
    cmovl esi, edx

    neg edx
    cmp edi, edx
    cmovg edi, edx

    ; alpha - esi
    ; beta - edi

    ; depth
    mov ecx, r13d

    ; ply count
    xor edx, edx
    call alpha_beta

    ; check for search failure
    cmp edx, eax ; edx = 0
    jo .end_search

    mov edx, MIN_EVAL

    cmp eax, esi ; score <= alpha
    jnle .no_aspiration_fail_low

    cmp eax, edx ; score != MIN_EVAL
    je .no_aspiration_fail_low

    ; fail low
    shl ebp, 1
    mov esi, eax
    sub esi, ebp
    jmp .do_search
.no_aspiration_fail_low:
    neg edx
    cmp eax, edi ; score >= beta
    jnge .no_aspiration_fail_high

    cmp eax, edx ; score != MAX_EVAL
    je .no_aspiration_fail_high

    ; fail high
    shl ebp, 1
    lea edi, [rax + rbp]
    jmp .do_search
.no_aspiration_fail_high:
    ; update best move and last score
    mov r14d, eax
    movzx r15d, word [rbx + Search.ply_data + PlyData.best_move]

%ifdef EXPORT_SYSV
    test r12d, r12d
    jz .no_search_print_info

    mov rdi, rbx
    mov esi, r13d
    mov edx, r15d
    mov ecx, eax

    push r11
    push rbp
    mov rbp, rsp
    and rsp, -16
    call search_print_info_sysv
    leave
    pop r11

.no_search_print_info:
    cmp r13d, r11d
    jge .end_search

    ; time_up clobbers r11 due to syscall
    push r11
    mov rdx, qword [rbx + Search.min_search_time]
    call time_up
    pop r11
%else
    mov rdx, qword [rbx + Search.min_search_time]
    call time_up
%endif
    jna .iterative_deepening_head
.end_search: 
    mov ebx, r15d
    ret

; rsp + 8 - search moves
; r14 - end of search moves
sort_search_moves:
    ; rax - outer loop counter
    push MovePlus_size
    pop rax
.outer_loop_head:
    cmp eax, r14d
    jae .end

    mov ecx, dword [rsp + rax + 8]
    mov edi, eax
.inner_loop_head:
    mov edx, dword [rsp + rdi + 8 - MovePlus_size]
    mov esi, edx
    mov si, -1
    ; or esi, 0FFFFh
    cmp ecx, esi
    jle .inner_loop_end

    mov dword [rsp + rdi + 8], edx
    sub edi, MovePlus_size
    jnz .inner_loop_head
.inner_loop_end:
    mov dword [rsp + rdi + 8], ecx
    add eax, MovePlus_size
    jmp .outer_loop_head
.end:
    ret

struc ABLocals
    .hash:
        resq 1
    .ordered_moves:
        resd 1
    .first_quiet:
        resd 1
    .best_eval:
        resd 1
    .best_move:
        resd 1 ; lower bits are ignored
    .alpha:
        resd 1
    .bound:
        resb 1
    .flags:
        resb 1
endstruc

IS_CHECK_FLAG equ 0001b
IMPROVING_FLAG equ 0010b
PV_NODE_FLAG equ 0100b
F_PRUNE_FLAG equ 1000b

IMPROVING_FLAG_INDEX equ 1

%if ABLocals_size > 128
%error "Alpha-Beta locals too large"
%endif


; search - rbx
; depth - rcx
; ply - rdx
; alpha - rsi
; beta - rdi
; preserves all general purpose registers except rcx and rax
alpha_beta:
    push r15
    push r14
    push r13
    push r12
    push r11
    push r10
    push r9
    push r8
    push rdi
    push rsi
    push rdx
    push rcx
    push rbp
    mov rbp, rsp
    sub rsp, 1024 + 128

    ; could by replaced by dword since upper bits don't actually
    ; do anything for playing strength.
    inc qword [rbx + Search.nodes]

    ; r13 - ply data
    mov rcx, qword [rbp + 16] ; ply count
    lea r13, [rbx + Search.ply_data + rcx * PlyData_size]

    ; clear non-hash locals
    vxorps xmm0, xmm0, xmm0
    vmovups yword [rbp - 128 + 8], ymm0

    ; rsi - current position
    mov rsi, qword [rbx]

    ; determine if we are in check
    ; preserves rsi
    call board_is_check

    ; IS_CHECK_FLAG = 1
    mov byte [rbp - 128 + ABLocals.flags], al

    ; check extension
    movzx ecx, al
    add dword [rbp + 8], ecx
    
    ; rdi - moves
    mov rdi, rsp
    mov r14, rdi

    ; generate moves
    ; preserves rsi
    call gen_moves

    ; find legal moves
    
    ; calculate the number of moves
    ; r15 - end of moves
    mov r15, rdi

    ; r14 - number of moves
    sub r14, rdi ; negative number
    jz .no_legal_moves
    sar r14, 2

    ; r11 - loop counter - counts towards zero
    mov r11, r14

    neg r14d
.find_legal_move_head:
    movzx edx, word [r15 + 4 * r11]

    call game_make_move
    jnc .legal_move_found

    inc r11
    jnz .find_legal_move_head

.no_legal_moves:
    ; no legal moves
    xor eax, eax
    test byte [rbp - 128 + ABLocals.flags], IS_CHECK_FLAG
    jz .stalemated
    mov eax, MIN_EVAL
.stalemated:
.fifty_move_draw:
    jmp .end

.legal_move_found:
    ; Unmake move
    add qword [rbx], -Board_size
    add rsi, -Board_size

    ; store as best move
    mov edx, dword [r15 + 4 * r11]
    mov word [r13 + PlyData.best_move], dx

    ; check 50 move rule
    ; rsi is now at board + Board_size
    xor eax, eax
    cmp byte [rsi + Board.fifty_moves], 100
    jge .fifty_move_draw

    ; rdi - position to search
    mov rdi, rsi

    ; repeating positions remaining before draw
    mov eax, 2
.reptition_loop_head:
    ; check for 50 move reset
    cmp byte [rdi + Board.fifty_moves], 0
    je .repetition_loop_end

    ; previous position
    add rdi, -Board_size

    mov ecx, 115
    repe cmpsb

    ; reset rdi and rsi without affecting flags
    lea rsi, [rsi + rcx - 115]
    lea rdi, [rdi + rcx - 115]

    jne .reptition_loop_head

    dec eax
    jnz .reptition_loop_head
    jmp .end
.repetition_loop_end:

    ; determine if this is a pv node
    mov edx, dword [rbp + 32]
    sub edx, dword [rbp + 24]
    dec edx

    jz .no_pv_node
    or byte [rbp - 128 + ABLocals.flags], PV_NODE_FLAG
.no_pv_node:

    ; check if we should stop
%ifdef EXPORT_SYSV
    test byte [RUNNING], 1
    jz .stop_search
%elif NUM_THREADS > 1
    test byte [RUNNING_WORKER_THREADS], 80h
    jz .stop_search
%endif
    ; nodes % 4096
    test dword [rbx + Search.nodes], 0FFFh
    jnz .no_stop_search

    mov rdx, qword [rbx + Search.max_search_time]
    call time_up
    ja .stop_search
.no_stop_search:

    ; probe the tt

    ; hash the position
    mov rsi, qword [rbx]
    mov eax, dword [rsi + Board.side_to_move]
    and eax, 00FFFFFFh
    vmovd xmm0, eax

    mov eax, 12
.hash_loop_head:
    vaesenc xmm0, xmm0, oword [rsi + Board.pieces + 8 * rax]
    dec eax
    jns .hash_loop_head

    vmovq rdx, xmm0
    mov qword [rbp - 128 + ABLocals.hash], rdx

    ; load tt_entry
    mov rcx, rdx

%ifndef EXPORT_SYSV
    and rdx, qword [TT_MASK]
    lea rax, [TT_MEM]
%else
    and rdx, qword [TT_MASK]
    mov rax, qword [TT_PTR]
%endif

    mov rax, qword [rax + rdx * 8]

    test rax, rax
    jz .tt_miss


    ; check the hashes
    shr rcx, 48 ; hash
    mov rdx, rax
    shr rdx, 48 ; tt hash
    cmp ecx, edx
    jne .tt_miss

    ; find the index of the entry

    ; rdi - first move
    mov rdi, rsp
    ; ecx - number of moves
    mov ecx, r14d

    ; clear score - scores are all zero since they came from movegen
    mov r12, rax ; TT entry
    movzx eax, ax
    repne scasd

    ; move not found
    jne .tt_miss

    ; check that the move is legal
    mov r11, rdi ; index + 4

    mov edx, eax
    call game_make_move
    jc .tt_miss

    ; Unmake move
    add qword [rbx], -Board_size

    ; Check if not qsearch
    cmp dword [rbp + 8], 0
    jg .tt_move_order

    ; If qsearch, check if move is noisy
    test r12d, (CAPTURE_FLAG | PROMO_FLAG) << 12
    jz .no_tt_move_order

.tt_move_order:
    ; swap the move with the first move

    mov ecx, dword [rsp]
    mov edx, dword [r11 - 4]

    mov dword [rsp], edx
    mov dword [r11 - 4], ecx

    ; set the number of ordered moves
    inc dword [rbp - 128 + ABLocals.ordered_moves]

.no_tt_move_order:

    ; get the depth of the tt entry
    mov rdx, r12
    mov eax, edx
    shr rdx, 34
    and edx, (1 << 14) - 1

    ; check for cutoff
    cmp edx, dword [rbp + 8]
    jnge .no_tt_cutoff

    test byte [rbp - 128 + ABLocals.flags], PV_NODE_FLAG
    jnz .no_tt_cutoff

    ; tt cutoffs
    sar eax, 16 ; eval
    shr r12, 32 ; bound
    test r12b, 11b
    jz .no_tt_cutoff ; No bound
    jpe .end ; Exact bound

    ; upper or lower bound
    test r12b, BOUND_LOWER
    jz .tt_upper_bound

    ; lower bound - check against beta
    cmp eax, dword [rbp + 32]
    jge .end
    jmp .tt_end
.tt_upper_bound:
    ; check against alpha
    cmp eax, dword [rbp + 24]
    jnle .tt_end

    ; would reduce the number of non-short jumps required,
    ; but is larger after compression
; .tt_cutoff: 
    jmp .end 
.tt_miss:
    ; iir
    ; This gives slightly different results if depth is negative but
    ; it does not matter
    cmp dword [rbp + 8], 4
    adc dword [rbp + 8], -1

;     cmp dword [rbp + 8], 3
;     jng .no_iir
;     dec dword [rbp + 8]
; .no_iir:
.tt_end:
.no_tt_cutoff:
.no_tt_probe:

    ; get the static evaluation
    mov rsi, qword [rbx]
    call evaluate

    ; store the static eval
    mov word [r13 + PlyData.static_eval], ax

    ; calculate the improving variable
    cmp dword [rbp + 16], 2
    jnae .not_improving
    cmp ax, word [r13 - 2 * PlyData_size + PlyData.static_eval]
    jng .not_improving

    or byte [rbp - 128 + ABLocals.flags], IMPROVING_FLAG
.not_improving:

    ; Null move pruning
    ; check depth
    mov ecx, dword [rbp + 8]
    cmp ecx, 0
    jng .no_null_move

    ; check if we are in check or in a pv node
    test byte [rbp - 128 + ABLocals.flags], IS_CHECK_FLAG | PV_NODE_FLAG
    jnz .no_null_move

    ; check that the static eval exceeds beta
    ; eax - static eval - beta
    sub eax, dword [rbp + 32]
    jnge .no_null_move

    ; static null move pruning
    ; check depth
    cmp ecx, 7
    jnle .no_static_nmp

    ; set margin for static nmp

    imul edx, ecx, STATIC_NULL_MOVE_MARGIN

    cmp eax, edx
    mov eax, dword [rbp + 32] ; beta
    jge .end
.no_static_nmp:
    ; check depth
    cmp ecx, 3
    jnge .no_null_move

    ; null move pruning
    ; the value of edx is 0
    xor edx, edx
    call game_make_move

    ; call alpha beta
    ; edx - ply count
    mov edx, dword [rbp + 16]
    inc edx

    ; ecx - reduced depth
    mov ecx, dword [rbp + 8]
    imul esi, ecx, 54
    add esi, 684 + 256 ; + 256 since formula is depth - r - 1

    test byte [rbp - 128 + ABLocals.flags], IMPROVING_FLAG
    jz .nmp_not_improving
    sub esi, 133

.nmp_not_improving:
    sar esi, 8
    sub ecx, esi

    ; rsi - -beta
    mov esi, dword [rbp + 32]
    neg esi

    ; edi - -beta + 1
    lea edi, [rsi + 1]

    call alpha_beta

    ; unmake move
    add qword [rbx], -Board_size

    neg eax
    jo .end

    ; beta cutoff
    cmp eax, dword [rbp + 32]
    jge .end
.no_null_move:

    ; order the noisy moves
    mov eax, dword [rbp - 128 + ABLocals.ordered_moves]
    mov rsi, qword [rbx]

    ; r8 - pieces
    mov r8, rsi
    test byte [rsi + Board.side_to_move], 1
    jz .order_noisy_white_move

    xor r8, 48

.order_noisy_white_move:

    ; sort the moves by flags

    lea r11, [rsp + 4 * rax] ; moves to sort

    ; r12 - number of moves
    mov r12d, r14d
    sub r12d, eax

    ; noisy move ordering assumes the existance of at least
    ; one move to order
    jz .order_noisy_no_moves

    call sort_moves_flags

    ; find first non-promotion
    xor ecx, ecx
.order_noisy_find_non_promo_head:
    test byte [r11 + 4 * rcx + 1], PROMO_FLAG << 4
    jz .order_noisy_non_promo

    inc ecx
    cmp ecx, r12d
    jne .order_noisy_find_non_promo_head

    ; no quiet moves, just sort
    jmp .order_noisy_sort_noisy
.order_noisy_non_promo:
    ; r11 - first non-promo move
    lea r11, [r11 + 4 * rcx]
    add dword [rbp - 128 + ABLocals.ordered_moves], ecx

    ; r12 - number of non-promo moves
    sub r12d, ecx

    ; count number of captures
.order_noisy_find_quiet_head:
    ; check if last move was noisy
    test byte [r11 + 4 * r12 - 4 + 1], (PROMO_FLAG | CAPTURE_FLAG) << 4
    jnz .order_noisy_sort_noisy
    
    dec r12d
    jnz .order_noisy_find_quiet_head

    ; sorting zero captures
.order_noisy_sort_noisy:
    add dword [rbp - 128 + ABLocals.ordered_moves], r12d
    call sort_moves_mvvlva
    

.order_noisy_no_moves:
    ; ecx - static eval
    movsx ecx, word [r13 + PlyData.static_eval]

    ; futility pruning
    ; edx - depth
    mov edx, dword [rbp + 8]

    cmp edx, 7
    jnle .no_fprune

    test byte [rbp - 128 + ABLocals.flags], IS_CHECK_FLAG | PV_NODE_FLAG
    jnz .no_fprune
    
    ; depth + improving
    xor esi, esi

    ; bt on memory is not that slow with imm
    bt dword [rbp - 128 + ABLocals.flags], IMPROVING_FLAG_INDEX
    adc esi, edx
.fprune_not_improving:

    ; set a minimum of 1
    mov eax, 1
    cmp esi, eax
    cmovl esi, eax

    imul esi, esi, F_PRUNE_MARGIN

    ; check if margin + static_eval is less than alpha
    add esi, ecx
    cmp esi, dword [rbp + 24]
    jnle .no_fprune

    or byte [rbp - 128 + ABLocals.flags], F_PRUNE_FLAG
.no_fprune:
    ; eax - best eval
    mov eax, MIN_EVAL

    ; stand pat in qsearch
    ; edx contains the depth
    cmp edx, 0
    cmovle eax, ecx
    mov dword [rbp - 128 + ABLocals.best_eval], eax

    ; beta cutoff
    cmp eax, dword [rbp + 32]
    jge .end

    ; check alpha
    mov dl, BOUND_UPPER
    mov ecx, dword [rbp + 24]
    cmp eax, ecx
    jng .initial_eval_no_alpha_raise

    ; exceeded alpha
    mov ecx, eax
    mov dl, BOUND_EXACT
.initial_eval_no_alpha_raise:

    ; set locals
    mov dword [rbp - 128 + ABLocals.alpha], ecx
    mov byte [rbp - 128 + ABLocals.bound], dl

    mov edx, dword [rbp - 128 + ABLocals.ordered_moves]
    mov dword [rbp - 128 + ABLocals.first_quiet], edx

    ; r15 - loop counter
    xor r15d, r15d
.main_search_loop_head:
    ; check if moves should be ordered
    mov edx, dword [rbp - 128 + ABLocals.ordered_moves]
    cmp r15d, edx
    jne .main_search_no_order_moves

    ; order the quiet moves

    ; all moves are now ordered
    mov dword [rbp - 128 + ABLocals.ordered_moves], r14d

    ; check depth for qsearch
    cmp dword [rbp + 8], 0
    jng .main_search_end

    ; order quiet moves
    xor esi, esi
.order_killer_moves_head:
    ; eax - killer move
    movzx eax, word [r13 + PlyData.kt + 2 * rsi]
    test eax, eax
    jz .order_killer_moves_end

    ; rdi - unordered moves
    lea rdi, [rsp + 4 * rdx]

    ; ecx - number of unordered moves
    mov ecx, r14d
    sub ecx, edx
    jz .order_killer_moves_tail

    ; FIXME: Currently the score are all zero, but this may change
    repne scasd
    jne .order_killer_moves_tail

    ; swap the moves
    mov ecx, dword [rsp + 4 * rdx]
    mov dword [rdi - 4], ecx
    mov dword [rsp + 4 * rdx], eax
    
    inc edx ; increment ordered moves
.order_killer_moves_tail:
    inc esi
    cmp esi, 2
    jne .order_killer_moves_head
.order_killer_moves_end:
    
    ; sort moves by history
    lea r11, [rsp + 4 * rdx]
    mov r12d, r14d
    sub r12d, edx

    ; load history
    lea r8, [rbx + Search.white_history]
    mov rsi, qword [rbx]
    test byte [rsi + Board.side_to_move], 1
    jz .order_quiet_white_moves

    add r8, Search.black_history - Search.white_history
.order_quiet_white_moves:
    call sort_moves_history

.main_search_no_order_moves:
    ; load the current move
    movzx r12d, word [rsp + 4 * r15]


    ; DEBUG: check that the move is noisy in qsearch
%ifdef DEBUG
    cmp dword [rbp + 8], 0
    jnle .debug_not_quiescence

    test r12d, (PROMO_FLAG | CAPTURE_FLAG) << 12
    jnz .debug_noisy
    int3
.debug_not_quiescence:
.debug_noisy:
%endif
    ; delta pruning
    ; check that futility pruning is enabled
    test byte [rbp - 128 + ABLocals.flags], F_PRUNE_FLAG
    jz .no_delta_prune

    ; check that we are in qsearch
    cmp dword [rbp + 8], 0
    jnle .no_delta_prune

    ; edi - eval
    movsx edi, word [r13 + PlyData.static_eval]
    add edi, DELTA_BASE

    ; add improving bonus
    test byte [rbp - 128 + ABLocals.flags], IMPROVING_FLAG
    jz .delta_prune_not_improving

    add edi, DELTA_IMPROVING_BONUS
.delta_prune_not_improving:

    ; rsi - piece values
    lea rsi, [DELTA_PRUNE_PIECE_VALUES]

    ; edx - move
    mov edx, r12d

    ; get the promo score
    test dh, PROMO_FLAG << 4
    jz .delta_prune_no_promo

    mov ecx, edx
    shr ecx, 12
    and ecx, 11b

    movzx ecx, word [rsi + 2 * rcx + 2]
    add edi, ecx
.delta_prune_no_promo:

    ; get the capture score

    ; r8 - enemy pieces
    mov r8, qword [rbx]
    test byte [r8 + Board.side_to_move], 1
    jnz .delta_prune_black

    add r8, 48
.delta_prune_black:
    shr edx, 6

    call board_get_piece
    cmp al, 0
    jl .delta_prune_no_capture

    movzx ecx, word [rsi + 2 * rax]
    add edi, ecx
.delta_prune_no_capture:
    cmp edi, dword [rbp - 128 + ABLocals.alpha]
    jle .main_search_tail
.no_delta_prune:
    ; make the move
    mov edx, r12d
    call game_make_move
    jc .main_search_tail

    ; rsi is a pointer to the current board
    call board_is_check

    ; r8 - gives check
    movzx r8d, al

    ; futility pruning
    test al, al
    jnz .no_fprune_move
    test r12d, (PROMO_FLAG | CAPTURE_FLAG) << 12
    jnz .no_fprune_move
    test byte [rbp - 128 + ABLocals.flags], F_PRUNE_FLAG
    jz .no_fprune_move

    ; prune
    add qword [rbx], -Board_size
    jmp .main_search_tail
.no_fprune_move:
    ; edi - -alpha
    mov edi, dword [rbp - 128 + ABLocals.alpha]
    neg edi

    ; PVS Search
    ; qsearch
    cmp dword [rbp + 8], 0
    jle .pvs_search_full

    ; no best move
    cmp dword [rbp - 128 + ABLocals.best_move], 0
    je .pvs_search_full

    ; lmr search
    ; ecx - depth - 1
    ; edx - depth
    ; esi - - alpha - 1
    mov edx, dword [rbp + 8]
    lea ecx, [rdx - 1]
    lea esi, [rdi - 1]

    ; depth
    cmp edx, 2
    jnge .no_lmr_reduction

    ; move num
    cmp r15d, 3
    jnge .no_lmr_reduction

    ; non-pv node and is check
    test byte [rbp - 128 + ABLocals.flags], IS_CHECK_FLAG
    jnz .no_lmr_reduction

    ; quiet move
    test r12d, (CAPTURE_FLAG | PROMO_FLAG) << 12
    jne .no_lmr_reduction

    ; gives check
    test r8d, r8d
    jnz .no_lmr_reduction

    ; calculate lmr depth
    ; 73 + depth * 28 + i * 29
    imul eax, edx, 28
    imul edx, r15d, 29
    lea eax, [rax + rdx + 73]

    ; decrease reduction if improving
    test byte [rbp - 128 + ABLocals.flags], IMPROVING_FLAG
    jz .lmr_not_improving
    sub eax, 129
.lmr_not_improving:
    ; divide by 256
    sar eax, 8
    sub ecx, eax

    cmp ecx, 1
    jge .no_history_leaf_pruning

    test byte [rbp - 128 + ABLocals.flags], PV_NODE_FLAG
    jnz .no_history_leaf_pruning

    ; history leaf pruning
    ; lead the history tables
    lea rax, [rbx + Search.white_history]
    mov rcx, qword [rbx]

    ; This check occurs after the move has alreay been made,
    ; so we are actually testing if it is currently black's move.
    test byte [rcx + Board.side_to_move], 1
    jnz .history_leaf_white

    add rax, Search.black_history - Search.white_history
.history_leaf_white:
    ; get the history of the move
    mov ecx, r12d
    and ecx, 0FFFh


    cmp qword [rax + 8 * rcx], 0
    mov ecx, 1 ; set the minimum lmr depth
    jnl .no_history_leaf_pruning

    ; prune
    add qword [rbx], -Board_size
    jmp .main_search_tail
.no_history_leaf_pruning:
.no_lmr_reduction:
    ; save lmr depth + 1
    lea r11d, [rcx + 1]

    ; ply + 1
    mov edx, dword [rbp + 16]
    inc edx

    call alpha_beta
    neg eax
    jo .pvs_search_failure

    ; possibly re-search

    ; check alpha
    cmp eax, dword [rbp - 128 + ABLocals.alpha]
    jng .pvs_end_search

    ; check beta
    cmp eax, dword [rbp + 32]
    jl .pvs_search_full

    ; check depth
    cmp r11d, dword [rbp + 8]
    je .pvs_end_search ; search with full depth already completed
.pvs_search_full:
    ; -beta
    mov esi, dword [rbp + 32]
    neg esi

    ; depth - 1
    or ecx, -1
    add ecx, dword [rbp + 8]

    ; ply + 1
    mov edx, dword [rbp + 16]
    inc edx

    call alpha_beta
    neg eax
    jno .pvs_end_search
.pvs_search_failure:
    ; unmake move
    add qword [rbx], -Board_size
    jmp .end
.pvs_end_search:

    ; unmake move
    add qword [rbx], -Board_size


    ; check best move
    cmp eax, dword [rbp - 128 + ABLocals.best_eval]
    jng .no_new_best_move

    ; update best move and eval
    mov dword [rbp - 128 + ABLocals.best_move], r12d
    mov dword [rbp - 128 + ABLocals.best_eval], eax

    ; check against beta
    cmp eax, dword [rbp + 32]
    jnge .no_beta_cutoff

    ; beta cutoff
    mov byte[rbp - 128 + ABLocals.bound], BOUND_LOWER

    ; move ordering for quiet moves
    test r12d, (CAPTURE_FLAG | PROMO_FLAG) << 12
    jnz .beta_cutoff_noisy

    ; update killer table
    ; edx - copy of move
    mov edx, r12d
    shl r12d, 16 ; temp
    shld dword [r13 + PlyData.kt], r12d, 16


    ; load history table
    lea r8, [rbx + Search.white_history]
    mov rsi, qword [rbx]
    test byte [rsi + Board.side_to_move], 1
    jz .decrease_white_history

    add r8, Search.black_history - Search.white_history

.decrease_white_history:

    ; beta cutoff for history
    ; eax - depth
    mov eax, dword [rbp + 8]
    imul eax, eax

    ; decrease history of searched quiet moves
    mov edi, dword [rbp - 128 + ABLocals.first_quiet]
.decrease_history_head:
    cmp edi, r15d
    jae .decrease_history_end

    ; get index
    mov esi, dword [rsp + 4 * rdi]
    and esi, 0FFFh

    ; subtract depth
    sub qword [r8 + 8 * rsi], rax
    inc edi
    jmp .decrease_history_head
.decrease_history_end:
    ; increase history of move causing cutoff
    and edx, 0FFFh
    add qword [r8 + 8 * rdx], rax

.beta_cutoff_noisy:
    jmp .main_search_end
.no_beta_cutoff:
.no_new_best_move:
    ; check alpha
    cmp eax, dword [rbp - 128 + ABLocals.alpha]
    jng .no_alpha_improvement

    ; update bound and alpha
    mov byte [rbp - 128 + ABLocals.bound], BOUND_EXACT
    mov dword [rbp - 128 + ABLocals.alpha], eax
.no_alpha_improvement:
.main_search_tail:
    inc r15d
    cmp r15d, r14d
    jne .main_search_loop_head
.main_search_end:
    ; eax - best eval
    mov eax, dword [rbp - 128 + ABLocals.best_eval]

    ; store tt and best move
    mov edx, dword [rbp - 128 + ABLocals.best_move]
    test edx, edx
    jz .no_store_tt

    ; store best move
    mov word [r13 + PlyData.best_move], dx

    ; load hash
    mov rdi, qword [rbp - 128 + ABLocals.hash]

    ; upper 16 bits of hash
    mov esi, 48
    bzhi rsi, rdi, rsi
    xor rsi, rdi

    ; load tt pointer and index

%ifndef EXPORT_SYSV
    lea r15, [TT_MEM]
    and rdi, qword [TT_MASK]
%else
    mov r15, qword [TT_PTR]
    and rdi, qword [TT_MASK]
%endif
    or rsi, rdx ; store move

    ; store eval
    mov edx, eax
    shl edx, 16
    or rsi, rdx

    ; depth
    xor edx, edx
    mov ecx, dword [rbp + 8]
    cmp ecx, 0
    cmovl ecx, edx

    shl ecx, 2
    or cl, byte [rbp - 128 + ABLocals.bound]
    shl ecx, 16
    shl rcx, 16
    or rsi, rcx

    ; store entry into tt
    mov qword [r15 + 8 * rdi], rsi
.no_store_tt:
    jmp .end
.stop_search:
    mov eax, NO_EVAL
.end:
    leave
    pop rcx
    pop rdx
    pop rsi
    pop rdi
    pop r8
    pop r9
    pop r10
    pop r11
    pop r12
    pop r13
    pop r14
    pop r15
    ret

; rbx - search
; rdx - time to search
; set by using ja
time_up:
    mov eax, CLOCK_GETTIME_SYSCALL

    push CLOCK_MONOTONIC
    pop rdi

    ; use red zone
    lea rsi, [rsp - 16]
    syscall

    ; calculate time used
    lodsq
    sub rax, qword [rbx + Search.start_tvsec]
    imul rcx, rax, 1_000_000_000

    lodsq
    sub rax, qword [rbx + Search.start_tvnsec]
    add rcx, rax

    ; compare
    cmp rcx, rdx
    ret
