NO_EVAL equ 80000000h
BOUND_NONE equ 00b
BOUND_LOWER equ 01b
BOUND_UPPER equ 10b
BOUND_EXACT equ 11b

F_PRUNE_MARGIN equ 320

section .rodata
DELTA_PRUNE_PIECE_VALUES:
    dw 256
    dw 832
    dw 832
    dw 1344
    dw 2496

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
    call root_search

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

    xor r12d, r12d ; temp
    call root_search

    push EXIT_SYSCALL
    pop rax
    xor edi, edi

    lock dec byte [RUNNING_WORKER_THREADS]

    syscall
%endif
; search - rbx
; time should be calculated before calling root_search
root_search:
    mov qword [rbx + Search.nodes], 0

    ; get static eval
    mov rsi, qword [rbx]
    push rsi
    call evaluate
    pop rsi
    mov word [rbx + Search.ply_data + PlyData.static_eval], ax

    ; memory for 256 moves, with stack alignment
    lea rdi, [rsp - 512 - 8]


    ; allocate memory for 256 SearchData's
    ; this overlaps with the memory for moves, but it is fine
    ; because the moves are never used afterwards
    sub rsp, 256 * SearchMove_size + 8

    ; make a copy
    push rdi ; Start of moves

    ; rsi is preserved by evaluate
    call gen_moves

    pop rsi ; Start of moves

    push rsp ; SearchData
    push rdi ; End of moves

    pop rbp ; End of moves
    pop rdi ; SearchData

    mov eax, MIN_EVAL - 1
.create_search_moves_head:
    stosw ; MIN_EVAL
    movsw ; move
    cmp esi, ebp ; upper bits don't matter
    jne .create_search_moves_head

    sub edi, esp ; upper bits don't matter
    cmp edi, SearchMove_size
    ja .more_than_one_move

    movzx eax, word [rsp + 2]
    jmp .return
.more_than_one_move:
    push rdi ; number of moves * SearchMove_size
    pop r15

    xor r13d, r13d ; r13d - depth
.iterative_deepening_head:
    xor r14d, r14d ; searched moves * SearchMove_size

    mov ebp, MIN_EVAL ; alpha
.root_search_moves_head:
    ; edx - move
    movzx edx, word [rsp + r14 + SearchMove.move]
    call game_make_move
    test al, al ; check legality
    jz .root_search_moves_tail

    ; alpha
    mov esi, MIN_EVAL

    ; beta
    mov edi, ebp
    neg edi

    ; depth
    mov ecx, r13d

    ; ply count
    push 1
    pop rdx
    call alpha_beta

    ; unmake move
    add qword [rbx], -Board_size

    ; check for search failure
    neg eax
    jo .end_search

    ; update score and alpha
    mov word [rsp + r14 + SearchMove.score], ax
    cmp eax, ebp
    cmovg ebp, eax

.root_search_moves_tail:
    add r14d, SearchMove_size
    cmp r14d, r15d
    jne .root_search_moves_head

    call sort_search_moves
    inc r13d

%ifdef EXPORT_SYSV
    test r12d, r12d
    jz .iterative_deepening_head

    mov rdi, rbx
    mov esi, r13d
    mov rdx, rsp

    push rbp
    mov rbp, rsp
    and rsp, -16
    call search_print_info_sysv
    leave
%endif

    jmp .iterative_deepening_head
.end_search: 
    call sort_search_moves

    movzx eax, word [rsp + SearchMove.move]
.return:
    add rsp, 1024 + 8
    ret

; rsp + 8 - search moves
; r14 - end of search moves
sort_search_moves:
    ; rax - outer loop counter
    push SearchMove_size
    pop rax
.outer_loop_head:
    cmp eax, r14d
    jae .end

    mov ecx, dword [rsp + rax + 8]
    mov edi, eax
.inner_loop_head:
    mov edx, dword [rsp + rdi + 8 - SearchMove_size]
    cmp cx, dx ; compare evals
    jle .inner_loop_end

    mov dword [rsp + rdi + 8], edx
    sub edi, SearchMove_size
    jnz .inner_loop_head
.inner_loop_end:
    mov dword [rsp + rdi + 8], ecx
    add eax, SearchMove_size
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
    sub rsp, 512 + 128

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

    mov eax, CLOCK_GETTIME_SYSCALL

    push CLOCK_MONOTONIC
    pop rdi

    lea rsi, [rbp - 128]
    syscall

    ; calculate time used
    lodsq
    sub rax, qword [rbx + Search.start_tvsec]
    imul rdx, rax, 1_000_000_000

    lodsq
    sub rax, qword [rbx + Search.start_tvnsec]
    add rdx, rax

    ; check if we have used too much time
    cmp rdx, qword [rbx + Search.search_time]
    ja .stop_search
.no_stop_search:
    ; could by replaced by dword since upper bits don't actually
    ; do anything for playing strength.
    inc qword [rbx + Search.nodes]

    ; clear non-hash locals
    vxorps xmm0, xmm0, xmm0
    vmovups yword [rbp - 128 + 8], ymm0

    call game_is_repetition
    movzx eax, al
    dec eax ; 0 if is repetition
    jz .end

    ; rsi - board - this is preserved by board_is_check
    mov rsi, qword [rbx]
    ; determine if we are in check
    call board_is_check

    ; IS_CHECK_FLAG = 1
    mov byte [rbp - 128 + ABLocals.flags], al

    ; check extension
    movzx ecx, al
    add dword [rbp + 8], ecx
    
    ; rdi - moves
    mov rdi, rsp
    mov r14, rdi

    ; preserves rsi
    call gen_moves

    ; find legal moves
    
    ; calculate the number of moves
    ; r15 - end of moves
    mov r15, rdi

    ; r14 - number of moves
    sub r14, rdi ; negative number
    jz .no_legal_moves
    sar r14, 1

    ; r13 - loop counter - counts towards zero
    mov r13, r14

    neg r14d
.find_legal_move_head:
    movzx edx, word [r15 + 2 * r13]

    ; sets rsi to current board if move was illegal,
    ; next board otherwise
    call game_is_move_legal
    test al, al
    jnz .legal_move_found

    inc r13
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
    ; check 50 move rule
    ; rsi is now at board + Board_size
    xor eax, eax
    cmp byte [rsi - Board_size + Board.fifty_moves], 100
    jge .fifty_move_draw

    ; probe the tt

    ; check depth
    cmp dword [rbp + 8], 0
    jng .no_tt_probe

    ; hash the position
    mov eax, dword [rsi - Board_size + Board.side_to_move]
    and eax, 00FFFFFFh
    vmovd xmm0, eax

    mov eax, 12
.hash_loop_head:
    vaesenc xmm0, xmm0, oword [rsi - Board_size + Board.pieces + 8 * rax]
    dec eax
    jns .hash_loop_head

    vmovq rdx, xmm0
    mov qword [rbp - 128 + ABLocals.hash], rdx

    ; load tt_entry
    mov rcx, rdx

%ifndef EXPORT_SYSV
    and rdx, TT_ENTRY_COUNT - 1
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
    repne scasw

    ; move not found
    jne .tt_miss

    ; check that the move is legal
    mov r13, rdi ; index + 2
    mov r12, rax ; TT entry

    movzx edx, ax ; TODO
    call game_is_move_legal
    test al, al
    jz .tt_miss

    ; swap the move with the first move

    ; read dwords, write words
    mov ecx, dword [rsp]
    mov edx, dword [r13 - 2]

    mov word [rsp], dx
    mov word [r13 - 2], cx

    ; set the number of ordered moves
    inc dword [rbp - 128 + ABLocals.ordered_moves]


    ; get the depth of the tt entry
    mov rdx, r12
    mov eax, edx
    shr rdx, 34 - 18
    sar edx, 18

    ; check for cutoff
    cmp edx, dword [rbp + 8]
    jnge .no_tt_cutoff

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
.tt_end:
.no_tt_cutoff:
.tt_miss:
.no_tt_probe:

    ; get the static evaluation
    mov rsi, qword [rbx]
    call evaluate

    ; r13 - ply data
    mov rcx, qword [rbp + 16] ; ply count
    lea r13, [rbx + Search.ply_data + rcx * PlyData_size]

    ; store the static eval
    mov word [r13 + PlyData.static_eval], ax

    ; determine if this is a pv node
    mov edx, dword [rbp + 32]
    sub edx, dword [rbp + 24]
    dec edx

    jz .no_pv_node
    or byte [rbp - 128 + ABLocals.flags], PV_NODE_FLAG
.no_pv_node:

    ; Null move pruning
    test byte [r13 + PlyData.no_nmp], 1
    jnz .no_null_move

    ; check depth
    cmp dword [rbp + 8], 4
    jnge .no_null_move

    ; check if we are in check or in a pv node
    test byte [rbp - 128 + ABLocals.flags], IS_CHECK_FLAG | PV_NODE_FLAG
    jnz .no_null_move

    ; null move pruning
    ; the value of edx is 0
    xor edx, edx
    call game_make_move

    ; call alpha beta
    mov byte [r13 + PlyData_size + PlyData.no_nmp], 1

    ; edx - ply count
    mov edx, dword [rbp + 16]
    inc edx

    ; ecx depth - r - 1
    ; where r = 3 if depth >= 6 and 2 otherwise
    mov ecx, dword [rbp + 8]
    cmp ecx, 6
    adc ecx, -4

    ; rdi - -beta
    mov esi, dword [rbp + 32]
    neg esi

    ; edi - -beta + 1
    lea edi, [rsi + 1]

    call alpha_beta

    mov byte [r13 + PlyData_size + PlyData.no_nmp], 0

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

    lea r11, [rsp + 2 * rax] ; moves to sort

    ; r12 - number of moves
    mov r12d, r14d
    sub r12d, eax

    ; noisy move ordering assumes the existance of at least
    ; one move to order
    jz .order_noisy_no_moves

    lea r15, [cmp_flags]
    call sort_moves

    ; find first non-promotion
    xor ecx, ecx
.order_noisy_find_non_promo_head:
    test byte [r11 + 2 * rcx + 1], PROMO_FLAG << 4
    jz .order_noisy_non_promo

    inc ecx
    cmp ecx, r12d
    jne .order_noisy_find_non_promo_head

    ; no quiet moves, just sort
    jmp .order_noisy_sort_noisy
.order_noisy_non_promo:
    ; r11 - first non-promo move
    lea r11, [r11 + 2 * rcx]
    add dword [rbp - 128 + ABLocals.ordered_moves], ecx

    ; r12 - number of non-promo moves
    sub r12d, ecx

    ; count number of captures
.order_noisy_find_quiet_head:
    ; check if last move was noisy
    test byte [r11 + 2 * r12 - 1], (PROMO_FLAG | CAPTURE_FLAG) << 4
    jnz .order_noisy_sort_noisy
    
    dec r12d
    jnz .order_noisy_find_quiet_head

    ; sorting zero captures
.order_noisy_sort_noisy:
    add dword [rbp - 128 + ABLocals.ordered_moves], r12d
    sub r15, cmp_flags - cmp_mvvlva
    call sort_moves
    

.order_noisy_no_moves:
    ; ecx - static eval
    movsx ecx, word [r13 + PlyData.static_eval]

    ; calculate the improving variable
    xor eax, eax ; eax - improving

    cmp dword [rbp + 16], 2
    jnae .not_improving
    cmp cx, word [r13 - 2 * PlyData_size + PlyData.static_eval]
    jng .not_improving

    or byte [rbp - 128 + ABLocals.flags], IMPROVING_FLAG
    inc eax
.not_improving:
    ; edx - depth
    mov edx, dword [rbp + 8]

    ; futility pruning
    ; ecx contains the static eval

    cmp edx, 3
    jnle .no_fprune

    test byte [rbp - 128 + ABLocals.flags], IS_CHECK_FLAG | PV_NODE_FLAG
    jnz .no_fprune
    
    ; depth + improving
    lea esi, qword [rdx + rax]

    ; set a minimum of 1
    mov al, 1
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
    lea rdi, [rsp + 2 * rdx]

    ; ecx - number of unordered moves
    mov ecx, r14d
    sub ecx, edx

    repne scasw
    jne .order_killer_moves_tail

    ; swap the moves
    mov ecx, dword [rsp + 2 * rdx]
    mov word [rdi - 2], cx
    mov word [rsp + 2 * rdx], ax
    
    inc edx ; increment ordered moves
.order_killer_moves_tail:
    inc esi
    cmp esi, 2
    jne .order_killer_moves_head
.order_killer_moves_end:
    
    ; sort moves by history
    lea r11, qword [rsp + 2 * rdx]
    mov r12d, r14d
    sub r12d, edx

    ; load history
    lea r8, [rbx + Search.white_history]
    mov rsi, qword [rbx]
    test byte [rsi + Board.side_to_move], 1
    jz .order_quiet_white_moves

    add r8, Search.black_history - Search.white_history
.order_quiet_white_moves:
    push r15
    lea r15, [cmp_history]
    call sort_moves
    pop r15
.main_search_no_order_moves:
    ; load the current move
    movzx r12d, word [rsp + 2 * r15]


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
    add edi, F_PRUNE_MARGIN

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

    ; check legality
    test al, al
    jz .main_search_tail

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
    mov ecx, dword [rbp + 8]
    mov edx, ecx
    dec ecx
    lea esi, [rdi - 1]

    ; depth
    cmp edx, 3
    jnge .no_lmr_reduction

    ; move num
    cmp r15d, 3
    jnge .no_lmr_reduction

    ; non-pv node and is check
    test byte [rbp - 128 + ABLocals.flags], IS_CHECK_FLAG | PV_NODE_FLAG
    jnz .no_lmr_reduction

    ; quiet move
    test r12d, (CAPTURE_FLAG | PROMO_FLAG) << 12
    jne .no_lmr_reduction

    ; gives check
    test r8d, r8d
    jnz .no_lmr_reduction

    ; calculate lmr depth
    ; depth / 4
    shr edx, 2

    ; + i / 8
    mov eax, r15d
    shr eax, 3
    add eax, edx

    ; reduction
    sub ecx, eax

    ; upper part of eax must be zero
    mov al, 1

    ; saturate at 1
    cmovle ecx, eax
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
    lea r8, qword [rbx + Search.white_history]
    mov rsi, qword [rbx]
    test byte [rsi + Board.side_to_move], 1
    jz .decrease_white_history

    add r8, Search.black_history - Search.white_history

.decrease_white_history:

    ; beta cutoff for history
    ; eax - depth
    mov eax, dword [rbp + 8]

    ; decrease history of searched quiet moves
    mov edi, dword [rbp - 128 + ABLocals.first_quiet]
.decrease_history_head:
    cmp edi, r15d
    jae .decrease_history_end

    ; get index
    mov esi, dword [rsp + 2 * rdi]
    and esi, 0FFFh

    ; subtract depth
    sub qword [r8 + 8 * rsi], rax
    inc edi
    jmp .decrease_history_head
.decrease_history_end:
    ; increase history of move causing cutoff
    imul eax, eax
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

    ; store tt
    mov edx, dword [rbp - 128 + ABLocals.best_move]

    test edx, edx
    jz .no_store_tt


    mov ecx, dword [rbp + 8]
    cmp ecx, 0
    jng .no_store_tt

    ; load hash
    mov rdi, qword [rbp - 128 + ABLocals.hash]

    ; upper 16 bits of hash
    mov esi, 48
    bzhi rsi, rdi, rsi
    xor rsi, rdi

    ; load tt pointer and index

%ifndef EXPORT_SYSV
    lea r15, [TT_MEM]
    and rdi, TT_ENTRY_COUNT - 1
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
