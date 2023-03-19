default rel
section .text

%ifdef EXPORT_SYSV
extern root_search_sysv
extern TT
extern TT_LEN
global start_sysv

start_sysv:
    jmp start
%endif

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

read_until_newline:
.loop:
    call read1
    cmp al, `\n`
    jne .loop

    ret

write8:
    push rdx

    push 1
    pop rax ; syscall 1
    mov edi, eax ; stdout fd

    push rsp
    pop rsi

    push 8
    pop rdx

    syscall

    pop rdx
    ret



start:
    ; set up shifts for movegen
    mov rax, 060A_0F11_0709_0108h
    push rax
    push rsp
    pop rsi

    lea rdi, [SHIFTS]
    push 8
    pop rcx

    xor eax, eax
.movegen_shifts_head:
    lodsb
    stosq
    loop .movegen_shifts_head

%if NUM_THREADS > 1
    push NUM_THREADS - 1
    pop rbx
.setup_threads:
    ; set up threads for lazy smp
    push MMAP_SYSCALL
    pop rax

    xor edi, edi

    ; map memory for stack
    mov esi, THREAD_STACK_SIZE
    push PROT_READ | PROT_WRITE
    pop rdx
    push MAP_PRIVATE | MAP_ANONYMOUS
    pop r10
    push -1
    pop r8
    xor r9d, r9d

    syscall

    ; set up stack to top of Search
    add rax, THREAD_STACK_SIZE - (MAX_BOARDS * Board_size + Search_size)

    ; Set up pointer to positions
    lea rbp, [rax + Search_size]
    mov qword [rax], rbp

    dec ebx
    jnz .setup_threads
    
%endif
    lea rdi, [TT_MEM]
    mov qword [TT], rdi
    mov qword [TT_LEN], TT_SIZE_BYTES / 8

;     syscall

    ; set up stack *SHOULD* already be zeroed
    sub rsp, (MAX_BOARDS * Board_size + Search_size)
    lea rdi, [rsp + Search_size]
    mov qword [rsp], rdi
    ; mov qword [rsp + Search.tt], rax

    ; set up startpos
    lea rsi, [STARTPOS]
    push 116
    pop rcx
    rep movsb

    ; wait for uci
    call read_until_newline

    ; write uciok

    mov rdx, `uciok  \n`
    call write8

.uci_loop_head:
    call read1
    cmp al, 'i'
    ja .not_isready_or_go
    jb .go

    ; isready
    mov rdx, `readyok\n`
    call write8
.read_until_newline_end_loop:
    call read_until_newline
    jmp .uci_loop_head
.not_isready_or_go:
    cmp al, 'q'
    jb .position
    ja .ucinewgame

    ; quit

    ; registers set up for read
    ; consume the 'uit' so that it doesn't get sent to bash
    xor eax, eax ; read syscall
    add edx, 2 ; count = 3
    syscall ; we don't care about the memory anymore
    
    ; exit
    push EXIT_SYSCALL
    pop rax
    xor edi, edi
    syscall
.ucinewgame:
    lea rdi, [TT_MEM]
    mov rcx, TT_SIZE_BYTES
    xor eax, eax
    rep stosb

    ; clear tt
    lea rdi, [rsp + Search.history]
    mov ecx, Search_size - Search.history
    rep stosb
    jmp .read_until_newline_end_loop
.go:
    mov eax, CLOCK_GETTIME_SYSCALL
    push CLOCK_MONOTONIC
    pop rdi
    lea rsi, [rsp + Search.start_time]
    syscall

    ; load w/b
    mov bl, 'w',
    mov rbp, qword [rsp]
    cmp byte [rbp + Board.side_to_move], 0
    je .go_white_move
    mov bl, 'b'
.go_white_move:


    ; find 'w/btime' - loop twice
    mov bh, 5
.go_find_time_head:
    call read1
    cmp al, bl
    jne .go_find_time_head

    movzx edx, bh
    call read

    xor ebp, ebp
.go_read_number_loop_head:
    call read1
    cmp al, ' ' ; check for space or new line
    jbe .go_read_number_end

    ; 1 byte displacement is required anyway, so do the subtraction here
    lea ebp, [rbp + 4 * rbp - ('0' / 2)]
    lea ebp, [rax + 2 * rbp] ; multiply by 2 and add digit
    jmp .go_read_number_loop_head
.go_read_number_end:
    push rbp

    dec bh
    jpo .go_find_time_head

    ; read until end of line
.go_read_until_newline_head:
    cmp al, `\n` ; might have been read when parsing inc
    je .go_finish_read
    call read1
    jmp .go_read_until_newline_head

.go_finish_read:
    ; temporary: link to non-asm
%ifdef EXPORT_SYSV
    mov ecx, 1
    pop rdx
    pop rsi
    mov rdi, rsp

    ; TEMP: align stack
    push rbp
    mov rbp, rsp
    and rsp, -16

    call root_search_sysv

    mov rsp, rbp
    pop rbp

    ; mov in ax
    push rax

    mov rdx, "bestmove"
    call write8

    pop rax
    mov ecx, 07070707h
    pdep ecx, eax, ecx
    add ecx, "a1a1"

    push ' '
    mov dword [rsp + 1], ecx ; add move
    test ah, PROMO_FLAG << 4

    mov dl, ' '
    jz .print_move_no_promo
    mov edx, "nbrq"
    shr eax, 9
    and al, 11000b
    xchg eax, ecx
    shr edx, cl
.print_move_no_promo:
    mov byte [rsp + 5], dl
    mov word [rsp + 6], ` \n`
    pop rdx
    call write8
%endif

    jmp .uci_loop_head

.position:
    ; read 2 * 8 bytes
    push 8
    pop rdx
    xor ebx, ebx
.position_read_startpos: ; read 16 bytes 'osition startpos'
    call read
    dec ebx
    jpe .position_read_startpos ; loops twice


    ; rbx - game
    push rsp
    pop rbx

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

    sub rsp, 512 ; allocate memory for moves
.position_make_moves:
    push 4 ; read the move
    pop rdx
    call read

    sub eax, 'a1a1' ; little endian, so the 'a' of the origin comes first

    ; ebp - squares of move
    mov ebp, 07070707h
    pext ebp, eax, ebp

    mov rsi, qword [rbx]
    push rsp
    pop rdi
    call gen_moves

.position_find_move_head:
    sub rdi, 2
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
    add rsp, 512
    jmp .uci_loop_head


