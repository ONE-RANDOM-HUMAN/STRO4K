section .data
uciok:
    db `uciok\n`
readyok:
    db `readyok\n`
print_info:
    db `info score cp - depth -\nbestmove `
.bestmove:
    db `     \n`
.end:


default rel
section .text

%ifndef EXPORT_SYSV
global _start

default rel
read1:
    push 1
    pop rdx

    ; read

; count - rdx
read:
    xor eax, eax ; read syscall
    push rax ; buf

    push rsp
    pop rsi ; pointer

    xor edi, edi ; stdin

    syscall

    pop rax
    ret

; rsi - format string - 1, where '-' indicates an integer
; rbp - format arguments
format_line:
    push 1
    pop rax ; syscall 1

    mov edi, eax ; stdout fd
    mov edx, eax ; size

.loop_head:
    inc rsi
    cmp byte[rsi], '-'
    jne .no_integer

    mov ebx, dword [rbp]
    add rbp, 8

    test ebx, ebx
    jns .positive

    syscall ; output '-'
    neg ebx
.positive:
    push rsi

    xor eax, eax
    xchg eax, ebx
    add edi, 9
.integer_head:
    cdq
    div edi

    add edx, '0'
    push rdx

    inc ebx
    test eax, eax
    jnz .integer_head

    inc eax ; syscall 1
    mov edi, eax ; stdout fd
    mov edx, eax ; size
.integer_write_head:
    push rsp
    pop rsi
    syscall

    pop rsi ; only 'free' register
    dec ebx
    jnz .integer_write_head

    pop rsi
    jmp .loop_head

.no_integer:
    syscall
    cmp byte[rsi], `\n`
    jne .loop_head

    ret



_start:
    ; wait for uci
.uci_read_loop:
    call read1
    cmp al, `\n`
    jne .uci_read_loop

    ; write uciok
    lea rsi, [uciok - 1]
    call format_line

    ; switch to the stack of the last thread
    mov rsp, THREAD_STACKS + NUM_THREADS * THREAD_STACK_SIZE - (MAX_BOARDS * Board_size) - Search_size

    ; set up startpos and pointer to positions
    lea rdi, [rsp + Search_size]
    mov qword [rsp], rdi

    lea rsi, [STARTPOS]

    push 116
    pop rcx
    rep movsb

.uci_loop_head:
    call read1
    cmp al, 'i'
    ja .not_isready_or_go
    jb .go

    ; isready
    lea rsi, [readyok - 1]
    call format_line
.isready_read_loop:
    call read1
    cmp al, `\n`
    jne .isready_read_loop

    jmp .uci_loop_head
.not_isready_or_go:
    cmp al, 'q'
    jb .position
    ja .ucinewgame

    ; quit

    ; registers set up for read
    ; consume the 'uit' so that it doesn't get sent to bash
.quit_read_loop:
    call read1
    cmp al, `\n`
    jne .quit_read_loop
    
    ; exit
.exit:
    push EXIT_SYSCALL
    pop rax
    xor edi, edi
    syscall

.ucinewgame:
.ucinewgame_read_loop:
    call read1
    cmp al, `\n`
    jne .ucinewgame_read_loop

    jmp .uci_loop_head
.go:
    mov eax, CLOCK_GETTIME_SYSCALL
    push CLOCK_MONOTONIC
    pop rdi
    lea rsi, [rsp + Search.start_time]
    syscall

    ; load w/b
    mov bl, 'w',
    mov rsi, qword [rsp]
    cmp byte [rsi + Board.side_to_move], 0
    je .go_white_move
    mov bl, 'b'
.go_white_move:


    ; find 'w/btime', 'w/binc'
.go_find_time_head_1:
    call read1
    cmp al, bl
    jne .go_find_time_head_1

    push 5
    pop rdx
    call read

    xor ebp, ebp
.go_read_number_loop_head_1:
    call read1
    cmp al, ' ' ; check for space or new line
    jbe .go_read_number_end_1

    imul ebp, ebp, 10
    lea ebp, [rbp + rax - '0']
    jmp .go_read_number_loop_head_1
.go_read_number_end_1:
    push rbp

.go_find_time_head_2:
    call read1
    cmp al, bl
    jne .go_find_time_head_2

    push 4
    pop rdx
    call read

    xor ebp, ebp
.go_read_number_loop_head_2:
    call read1
    cmp al, ' ' ; check for space or new line
    jbe .go_read_number_end_2

    imul ebp, ebp, 10
    lea ebp, [rbp + rax - '0']
    jmp .go_read_number_loop_head_2
.go_read_number_end_2:
    push rbp

    ; read until end of line
    cmp al, `\n` ; might have been read when parsing inc
    je .go_finish_read
.go_read_until_newline_head:
    call read1
    cmp al, `\n`
    jne .go_read_until_newline_head

.go_finish_read:
    mov dword [RUNNING_WORKER_THREADS], 8000_0000h | (NUM_THREADS - 1)
    and qword [SEARCH_RESULT], 0

%if NUM_THREADS > 1
    ; create threads
    ; rbx - search
    mov rbx, THREAD_STACKS + THREAD_STACK_SIZE - MAX_BOARDS * Board_size - Search_size
.create_thread_head:
    ; copy the boards
    lea rsi, [rsp + Search_size + 16]
    mov rcx, qword [rsp + 16]
    sub rcx, rsi

    lea rdi, [rbx + Search_size]
    rep movsb

    mov qword [rbx], rdi
    mov cl, 128
    rep movsb ; copy the current position

    ; clone the thread
    push CLONE_SYSCALL
    pop rax
    
    mov edi, CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | CLONE_THREAD

    ; new stack
    mov rsi, rbx
    syscall

    test eax, eax
    jnz .no_thread_search

    ; set infinite time
    or qword [rbx + Search.min_search_time], -1
    or qword [rbx + Search.max_search_time], -1

    call root_search

    lock dec dword [RUNNING_WORKER_THREADS]
    jmp _start.exit
.no_thread_search:
    
    add rbx, THREAD_STACK_SIZE
    cmp rbx, rsp
    jb .create_thread_head
%endif
    ; Calculate time
    pop rdx
    pop rsi

    mov rbx, rsp

    imul rcx, rsi, 84901
    imul rax, rdx, 575901
    add rcx, rax
    mov qword [rbx + Search.max_search_time], rcx

    imul rcx, rsi, 27098
    imul rax, rdx, 8979
    add rcx, rax
    mov qword [rbx + Search.min_search_time], rcx

    call root_search

.go_wait_for_threads:
    lock and dword [RUNNING_WORKER_THREADS], 7FFF_FFFFh
    jnz .go_wait_for_threads

    mov rdi, qword [rbx]
    mov ecx, dword [rbx + 4]
    lea rsi, [print_info - 1]

    mov ebx, 07070707h
    pdep eax, ecx, ebx
    add eax, "a1a1"

    mov dword [rsi + 1 + print_info.bestmove - print_info], eax

    mov dl, ' '
    test ch, PROMO_FLAG << 4
    jz .print_move_no_promo
    mov edx, "nbrq"
    shr ecx, 9

    and cl, 11000b
    shr edx, cl
.print_move_no_promo:
    mov byte [rsi + 1 + print_info.bestmove - print_info + 4], dl

    push rdi ; depth - only lower 32 bits matters
    sar rdi, 48
    push rdi ; score

    push rsp
    pop rbp

    ; interleaving pops is slightly smaller
    call format_line
    pop rax

    call format_line
    pop rax

    jmp .uci_loop_head

.position:
    ; read 2 * 8 bytes
    ; 'osition startpos'
    push 8
    pop rdx

    call read
    call read

    ; rbx - game
    mov rbx, rsp

    ; reset game to startpos
    lea rbp, [rsp + Search_size]
    mov qword [rbx], rbp

    ; check if there ane any moves
    call read1
    cmp al, `\n`
    je .uci_loop_head

    push 6 ; read 'moves '
    pop rdx
    call read

    sub rsp, 1024 ; allocate memory for moves
.position_make_moves:
    push 4 ; read the move
    pop rdx
    call read

    sub eax, 'a1a1' ; little endian, so the 'a' of the origin comes first

    ; ebp - squares of move
    mov ebp, 07070707h
    pext ebp, eax, ebp

    mov rsi, qword [rbx]
    mov rdi, rsp
    call gen_moves

.position_find_move_head:
    sub rdi, 4
    movzx edx, word [rdi]
    mov ecx, edx
    and ch, 00001111b
    cmp ecx, ebp
    je .position_move_found

    ; if we go past the end, then the move was illegal
    ; or there was a bug and it's just too bad
    jmp .position_find_move_head
.position_move_found:
    test dh, PROMO_FLAG << 4
    jz .position_promo_end

    ; dx must contain a knight promo because it is the last to be generated
    push rdx
    call read1
    pop rdx

    cmp al, 'n'
    je .position_knight_promo
    jb .position_bishop_promo
    or dh, 0010_0000b
    cmp al, 'q'
    jne .position_promo_end
.position_bishop_promo:
    or dh, 0001_0000b
.position_knight_promo:
.position_promo_end:
    call game_make_move

    call read1
    cmp al, `\n`
    jne .position_make_moves
.position_end:
    add rsp, 1024
    jmp .uci_loop_head

%endif
