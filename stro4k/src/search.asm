NO_EVAL equ 80000000h
BOUND_NONE equ 00b
BOUND_LOWER equ 01b
BOUND_UPPER equ 10b
BOUND_EXACT equ 11b

F_PRUNE_MARGIN equ 100
STATIC_NULL_MOVE_MARGIN equ 80
DELTA_BASE equ 287
SEE_PRUNE_MARGIN equ -72

section .rodata
PIECE_VALUES:
    dd 114
    dd 425
    dd 425
    dd 648
    dd 1246
    dd MAX_EVAL

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
    push qword [rbx] ; save current position
    push rbx

    mov r12d, esi
    mov r11d, edx
    call root_search
    mov eax, ebx

    pop rbx
    pop qword [rbx]

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
%ifdef EXPORT_SYSV
    ; Save in last plydata - should never be used
    mov qword [rbx + Search.ply_data + (MAX_BOARDS - 1) * PlyData_size], rsp
%endif
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
    mov ebp, 18

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
    mov edx, r14d

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
    .static_eval:
        resw 1
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
    jna .no_stop_search
.stop_search:
    ; TODO
%ifdef EXPORT_SYSV
    mov rsp, qword [rbx + Search.ply_data + (MAX_BOARDS - 1) * PlyData_size]
%else
    lea rsp, [rbx - 8]
%endif
    ; restore r15 - it is the first register to be pushed by alpha_beta
    ; after it is called by root_search.
    mov r15d, dword [rsp - 16]
    jmp root_search.end_search
.no_stop_search:

    ; get the static evaluation
    mov rsi, qword [rbx]
    call evaluate

    ; store the static eval
    mov word [r13 + PlyData.static_eval], ax
    mov word [rbp - 128 + ABLocals.static_eval], ax

    ; calculate the improving variable
    cmp dword [rbp + 16], 2
    jnae .not_improving
    cmp ax, word [r13 - 2 * PlyData_size + PlyData.static_eval]
    jng .not_improving

    or byte [rbp - 128 + ABLocals.flags], IMPROVING_FLAG
.not_improving:

    ; probe the tt

    ; hash the position
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

    ; give the first move the highest score
    mov word [rsp + 2], 7FFFh

    ; set the number of ordered moves
    inc dword [rbp - 128 + ABLocals.ordered_moves]

.no_tt_move_order:

    ; get the depth of the tt entry
    mov rdx, r12
    mov eax, edx
    shr rdx, 34
    and edx, (1 << 14) - 1

    ; tt cutoffs and static eval
    sar eax, 16 ; eval
    shr r12, 32 ; bound
    test r12b, 11b
    jz .tt_end ; No bound

    ; cx - static eval
    movsx ecx, word [r13 + PlyData.static_eval]
    cmovpe ecx, eax
    jpe .possible_tt_cutoff ; Exact bound

    ; upper or lower bound
    test r12b, BOUND_LOWER
    jz .tt_upper_bound

    ; lower bound - check against beta
    cmp eax, ecx
    cmovg ecx, eax

    cmp eax, dword [rbp + 32]
    jge .possible_tt_cutoff
    jmp .no_tt_cutoff
.tt_upper_bound:
    ; upper bound - check against alpha
    cmp eax, ecx
    cmovl ecx, eax

    cmp eax, dword [rbp + 24]
    jnle .no_tt_cutoff

    ; would reduce the number of non-short jumps required,
    ; but is larger after compression
.possible_tt_cutoff:
    ; check for cutoff
    cmp edx, dword [rbp + 8]
    jnge .no_tt_cutoff

    test byte [rbp - 128 + ABLocals.flags], PV_NODE_FLAG
    jnz .no_tt_cutoff
    jmp .end 
.no_tt_cutoff:
    mov word [rbp - 128 + ABLocals.static_eval], cx
    jmp .tt_end
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
.no_tt_probe:

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
    movsx eax, word [rbp - 128 + ABLocals.static_eval]
    sub eax, dword [rbp + 32]
    jnge .no_null_move

    ; save static_eval - beta
    mov r15d, eax

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
    xor edx, edx
    call game_make_move

    ; call alpha beta
    ; edx - ply count
    mov edx, dword [rbp + 16]
    inc edx

    ; ecx - reduced depth
    mov ecx, dword [rbp + 8]
    imul esi, ecx, 61
    lea esi, [rsi + 2 * r15 + 618 + 256] ; + 256 since formula is depth - r - 1

    test byte [rbp - 128 + ABLocals.flags], IMPROVING_FLAG
    jz .nmp_not_improving
    sub esi, 49

.nmp_not_improving:
    sar esi, 8
    sub ecx, esi

    ; rsi - -beta
    mov esi, dword [rbp + 32]
    neg esi

    ; edi - -beta + 1
    lea edi, [rsi + 1]

    call alpha_beta
    neg eax

    ; unmake move
    add qword [rbx], -Board_size

    ; beta cutoff
    cmp eax, dword [rbp + 32]
    jge .end
.no_null_move:

    ; order the noisy moves

    ; r8 - pieces
    mov r8, qword [rbx]
    cmp byte [r8 + Board.side_to_move], 0
    je .order_noisy_white_move

    xor r8, 48

.order_noisy_white_move:

    ; sort the moves by flags and mvvlva

    ; noisy move ordering assumes the existance of at least
    ; one move to order
    mov esi, dword [rbp - 128 + ABLocals.ordered_moves]
    cmp esi, r14d
    jae .order_noisy_no_moves

    ; edi - loop counter
.order_noisy_score_head:
    ; edx - move
    movzx edx, word [rsp + 4 * rsi + MovePlus.move]

    ; if move > CAPTURE_FLAG, then it is a capture or promo
    cmp edx, CAPTURE_FLAG << 12
    sbb dword [rbp - 128 + ABLocals.ordered_moves], -1

    ; eax - score
    xor eax, eax

    test dh, CAPTURE_FLAG << 4
    jz .order_noisy_non_capture

    ; eax - attacker, ecx - victim
    call board_get_move_pieces

    ; eax - (8 * victim + 1) - attacker
    neg eax
    lea eax, [rax + 8 * rcx + 8]
.order_noisy_non_capture:
    shr edx, 12
    mov ah, dl
    mov word [rsp + 4 * rsi + MovePlus.score], ax

    inc esi
    cmp esi, r14d
    jb .order_noisy_score_head
.order_noisy_no_moves:
    ; ecx - static eval
    movsx ecx, word [rbp - 128 + ABLocals.static_eval]

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
    mov ecx, dword [rbp - 128 + ABLocals.ordered_moves]
    cmp r15d, ecx
    jne .main_search_no_order_moves

    ; order the quiet moves

    ; all moves are now ordered
    mov dword [rbp - 128 + ABLocals.ordered_moves], r14d

    ; check depth for qsearch
    cmp dword [rbp + 8], 0
    jng .main_search_end

    ; order quiet moves
    ; sort moves by history and killers
    mov eax, dword [r13 + PlyData.kt]

    ; load history
    lea r8, [rbx + Search.white_history]
    mov rsi, qword [rbx]
    cmp byte [rsi + Board.side_to_move], 0
    je .order_quiet_white_moves

    add r8, Search.black_history - Search.white_history
.order_quiet_white_moves:

    ; there must be at least one quiet move or the search would have ended
.history_score_head:
    movzx edx, word [rsp + 4 * rcx + MovePlus.move]
    mov edi, edx
    and edi, 0FFFh

    movsx esi, word [r8 + 8 * rdi]

    ; killers
    mov edi, 07FFFh

    ; unrolling this loop compresses very well
    cmp dx, ax
    cmove esi, edi
    ror eax, 16
    dec edi

    cmp dx, ax
    cmove esi, edi
    ror eax, 16
    dec edi ; does nothing but is 1 byte smaller after compression

    mov word [rsp + 4 * rcx + MovePlus.score], si

    inc ecx
    cmp ecx, r14d
    jne .history_score_head
.history_score_end:

.main_search_no_order_moves:

    ; load the next move

    ; find the move with the highest score
    mov edi, r15d
    mov esi, edi
.find_highest_score_head:
    inc edi
    cmp edi, r14d
    je .find_highest_score_end

    movzx ecx, word [rsp + 4 * rdi + MovePlus.score]
    cmp cx, word [rsp + 4 * rsi + MovePlus.score]
    cmovg esi, edi

    jmp .find_highest_score_head
.find_highest_score_end:
    ; swap with the next move in the sequence
    mov edi, dword [rsp + 4 * rsi]
    mov ecx, dword [rsp + 4 * r15]
    mov dword [rsp + 4 * rsi], ecx
    mov dword [rsp + 4 * r15], edi

    movzx r12d, di

    cmp dword [rbp + 8], 7
    jnle .not_quiescence_no_see

    ; SEE pruning
    push r15 ; beta
    push r14 ; alpha
    push r13 ; eval
    lea r11, [PIECE_VALUES]

    ; make null move
    xor edx, edx
    call game_make_move

    ; r8 - pieces
    mov edx, r12d
    call board_get_move_pieces

    shr edx, 6
    xor edi, edi
    bts rdi, rdx
    push rdi

    ; r15d - beta
    xor r13d, r13d
    test ecx, ecx
    js .see_no_captured_piece

    ; no need to remove captured piece

    ; This can't be replaced by cmovns because that always performs the load
    mov edi, ecx
    mov r13d, dword [r11 + 4 * rdi]
.see_no_captured_piece:
    mov r15d, r13d ; r15d - beta

    ; edx - attacking piece square
    mov edi, eax
    xor eax, eax
    bts rax, r12

    ; remove attacking piece
    ; since board_area_attacked_by only takes the xor of white and black to
    ; calculate occ, we can always xor into white occ
    xor qword [r8 + 8 * rdi], rax
    xor qword [rsi + Board.colors], rax

    sub r13d, dword [r11 + 4 * rdi]
    mov r14d, r13d

.see_loop_head:
    xor byte [rsi + Board.side_to_move], 1

    mov rdi, qword [rsp]
    call board_area_attacked_by
    jz .see_fail_high

    blsi rax, rax

    xor qword [r10 + 8 * rdi], rax
    xor qword [rsi + Board.colors], rax

    add r13d, dword [r11 + 4 * rdi]

    ; check
    cmp r13d, r14d
    jle .see_fail_low

    ; update beta
    cmp r13d, r15d
    cmovle r15d, r13d



    xor byte [rsi + Board.side_to_move], 1

    mov rdi, qword [rsp]
    call board_area_attacked_by
    jz .see_fail_low

    blsi rax, rax

    xor qword [r10 + 8 * rdi], rax
    xor qword [rsi + Board.colors], rax

    sub r13d, dword [r11 + 4 * rdi]

    ; check beta
    cmp r13d, r15d
    jge .see_fail_high

    ; update alpha
    cmp r13d, r14d
    cmovge r14d, r13d

    jmp .see_loop_head
.see_fail_high:
    mov r14d, r15d
.see_fail_low:
    pop rdi
    mov edi, r14d

    pop r13
    pop r14
    pop r15
    add qword [rbx], -128

    test byte [rbp - 128 + ABLocals.flags], IS_CHECK_FLAG | PV_NODE_FLAG
    jnz .no_see_pruning

    xor esi, esi
    imul eax, dword [rbp + 8], SEE_PRUNE_MARGIN

    ; We can't just use SF since imul leaves it unspecified
    cmp esi, eax
    cmovnl esi, eax

    cmp edi, esi
    jl .main_search_tail
.no_see_pruning:

    cmp dword [rbp + 8], 0
    jnle .not_quiescence

    ; DEBUG: check that the move is noisy in qsearch
%ifdef DEBUG
    test r12d, (PROMO_FLAG | CAPTURE_FLAG) << 12
    jnz .debug_noisy
    int3
.debug_noisy:
%endif

    ; delta pruning
    ; check that futility pruning is enabled
    test byte [rbp - 128 + ABLocals.flags], F_PRUNE_FLAG
    jz .no_delta_prune

    ; edi - static eval + see
    movsx eax, word [rbp - 128 + ABLocals.static_eval]
    lea edi, [rdi + rax + DELTA_BASE]

    mov edx, r12d
    test dh, PROMO_FLAG << 4
    jz .delta_prune_no_promo

    shr edx, 12
    and edx, 11b
    add edi, dword [r11 + 4 * rdx + 4]
.delta_prune_no_promo:
    cmp edi, dword [rbp - 128 + ABLocals.alpha]
    jle .main_search_tail
.no_delta_prune:
.not_quiescence:
.not_quiescence_no_see:
    ; make the move
    mov edx, r12d
    call game_make_move
    jc .main_search_tail

    ; rsi is a pointer to the current board
    call board_is_check

    movzx r8d, al

    ; futility pruning
    ; is check
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
    ; edx - depth
    mov edx, dword [rbp + 8]

    ; depth
    cmp edx, 2
    jnge .no_lmr_reduction

    ; move num
    cmp r15d, 3
    jnge .no_lmr_reduction

    ; calculate lmr depth
    ; 106 + depth * 15 + i * 36
    imul eax, edx, 15
    imul ecx, r15d, 36
    lea eax, [rax + rcx + 106]

    ; decrease reduction if improving
    test byte [rbp - 128 + ABLocals.flags], IMPROVING_FLAG
    jz .lmr_not_improving
    sub eax, 152
.lmr_not_improving:
    ; divide by 256
    sar eax, 8

    ; edx - lmr_depth + 1
    sub edx, eax

    cmp edx, 2
    jge .no_history_leaf_pruning

    mov edx, 2 ; set the minimum lmr depth + 1

    ; non-pv node and is check
    test byte [rbp - 128 + ABLocals.flags], PV_NODE_FLAG | IS_CHECK_FLAG
    jnz .no_history_leaf_pruning

    ; quiet move
    test r12d, (CAPTURE_FLAG | PROMO_FLAG) << 12
    jnz .no_history_leaf_pruning

    ; gives check
    test r8d, r8d
    jnz .no_history_leaf_pruning


    ; history leaf pruning
    ; lead the history tables
    lea rax, [rbx + Search.white_history]
    mov rsi, qword [rbx]

    ; This check occurs after the move has alreay been made,
    ; so we are actually testing if it is currently black's move.
    cmp byte [rsi + Board.side_to_move], 0
    jne .history_leaf_white

    add rax, Search.black_history - Search.white_history
.history_leaf_white:
    ; get the history of the move
    mov ecx, r12d
    and ecx, 0FFFh

    ; this redudant REX prefix reduces compressed size by 2 bytes somehow
    db 40h
    cmp dword [rax + 8 * rcx], 0
    jnl .no_history_leaf_pruning

    ; prune
    add qword [rbx], -Board_size
    jmp .main_search_tail
.no_history_leaf_pruning:
.no_lmr_reduction:
    ; save lmr depth + 1
    mov r11d, edx

    ; ecx - lmr depth
    lea ecx, [rdx - 1]

    ; esi - -alpha - 1
    lea esi, [rdi - 1]

    ; ply + 1
    mov edx, dword [rbp + 16]
    inc edx

    call alpha_beta
    neg eax

    ; possibly re-search

    ; check alpha
    cmp eax, dword [rbp - 128 + ABLocals.alpha]
    jng .pvs_no_research

    ; check beta
    cmp eax, dword [rbp + 32]
    jl .pvs_search_full

    ; check depth
    cmp r11d, dword [rbp + 8]
    je .pvs_no_research ; search with full depth already completed
.pvs_search_full:
    ; -beta
    mov esi, dword [rbp + 32]
    neg esi

    ; depth - 1
    mov ecx, -1
    add ecx, dword [rbp + 8]

    ; ply + 1
    mov edx, dword [rbp + 16]
    inc edx

    call alpha_beta
    neg eax

.pvs_no_research:
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
    cmp byte [rsi + Board.side_to_move], 0
    je .decrease_white_history

    add r8, Search.black_history - Search.white_history

.decrease_white_history:

    ; beta cutoff for history
    ; eax - depth
    mov eax, dword [rbp + 8]
    imul eax, eax

    mov ecx, 2048
    cmp eax, ecx
    cmovg eax, ecx

    ; decrease history of searched quiet moves
    mov edi, dword [rbp - 128 + ABLocals.first_quiet]
.decrease_history_head:
    cmp edi, r15d
    jae .decrease_history_end

    ; get index
    mov esi, dword [rsp + 4 * rdi]
    and esi, 0FFFh

    mov ecx, eax
    imul ecx, dword [r8 + 8 * rsi]
    sar ecx, 11
    add ecx, eax

    ; subtract depth
    sub dword [r8 + 8 * rsi], ecx
    inc edi
    jmp .decrease_history_head
.decrease_history_end:
    ; increase history of move causing cutoff

    mov esi, edx
    and esi, 0FFFh

    mov ecx, eax
    imul ecx, dword [r8 + 8 * rsi]
    sar ecx, 11
    sub ecx, eax ; using negative increase improves compression

    sub dword [r8 + 8 * rsi], ecx

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
