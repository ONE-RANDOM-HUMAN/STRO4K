default rel
section .text

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


start:
    ; set up shifts for movegen
    mov rax, 060A_0F11_0709_0108h
    push rax
    push rsp
    pop rsi

    lea rdi, [SHIFTS]
    mov cl, 8

    xor eax, eax
.movegen_shifts_head:
    lodsb
    stosq
    dec cl
    jnz .movegen_shifts_head

%if NUM_THREADS > 1
.setup_threads:
    ; set up threads for lazy smp
    push MMAP_SYSCALL
    pop rax

    push NUM_THREADS - 1
    pop rbx
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
    lea rcx, [rax + Search_size]
    mov qword [rax], rcx

    dec ebx
    jnz .setup_threads
%endif
    ; set up stack
    sub rsp, (MAX_BOARDS * Board_size + Search_size)
    lea rcx, [rsp + Search_size]
    mov qword [rsp], rcx

    ; wait for uci
    call read_until_newline

    ; write uciok
    ; registers already set up for read
    inc edi ; stdout
    add edx, 5 ; 6 bytes

    mov rax, `uciok\n`
    push rax ; rsi is already pointing here

    mov eax, edi ; write syscall
    syscall
    pop rax

.uci_loop_head:
    call read1
    cmp al, 'i'
    jne .not_isready

    ; isready
    ; registers set up for read
    inc edi
    add edx, 7

    mov rax, `isready\n`
    push rax

    mov eax, edi ; write syscall
    syscall
    pop rax
    jmp .uci_loop_head
.not_isready:
    cmp al, 'p'
    jb .go
    je .position

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
.go:
.position:

