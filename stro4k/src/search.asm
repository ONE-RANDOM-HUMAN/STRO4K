default rel
section .text

%ifdef EXPORT_SYSV
    extern search_alpha_beta_sysv
    extern search_print_info_sysv
%endif

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

%ifdef EXPORT_SYSV
    mov rdi, rbx

    ; alpha
    mov esi, MIN_EVAL

    ; beta
    mov edx, ebp
    neg edx

    ; depth
    mov ecx, r13d

    ; ply count
    mov r8d, 1

    call search_alpha_beta_sysv
%endif
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

    test r12d, r12d
    jz .iterative_deepening_head
%ifdef EXPORT_SYSV
    mov rdi, rbx
    mov esi, r13d
    mov rdx, rsp

    call search_print_info_sysv
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






