default rel
section .text

%ifdef EXPORT_SYSV
    global sort_moves
    global cmp_history
    global cmp_flags
    global cmp_mvvlva
%endif

; board - rsi
; moves - r11
; move count - r12
; less function - r15
; preserves rbx, rbp, r8, r11, r12, r14, r15
sort_moves:
    push rbx

    ; r13 - inner loop counter
    mov r13d, 1
.outer_loop_head:
    cmp r13d, r12d
    jae .end

    movzx r10d, word [r11 + 2 * r13]
    mov r9d, r13d
.inner_loop_head:
    mov edx, r10d
    movzx edi, word [r11 + 2 * r9 - 2]
    mov ebx, edi

    ; compare moves
    call r15
    test al, al
    jz .inner_loop_end

    mov word [r11 + 2 * r9], bx
    dec r9d
    jnz .inner_loop_head
.inner_loop_end
    mov word [r11 + 2 * r9], r10w
    inc r13d
    jmp .outer_loop_head
.end
    pop rbx
    ret


; lhs - rdx
; rhs - rdi
; the performace of this is terrible, probably because of the fn call
cmp_flags:
    or edi, 0FFFh
    cmp edx, edi
    seta al
    ret

; lhs - rdx
; rhs - rdi
; r8 - history
cmp_history:
    mov eax, 0FFFh
    and edx, eax
    and edi, eax

    mov rdx, qword [r8 + 8 * rdx]
    cmp rdx, qword [r8 + 8 * rdi]
    setg al
    ret
    

; board - rsi
; lhs - rdx
; rhs - rdi
; pieces - r8
cmp_mvvlva:
    push rbx
    ; ebx - lhs attacker
    call board_get_piece ; lhs attacker
    mov ebx, eax
    shr edx, 6

    ; ebx - lhs attacker - 8 * lhs victim
    xor r8, 48 ; switch pieces, taking advantage of 128 byte alignment
    call board_get_piece ; lhs victim
    shl eax, 3 
    sub ebx, eax

    ; ebx - lhs attacker - rhs attacker - 8 * lhs victim
    xor r8, 48
    mov edx, edi
    call board_get_piece ; rhs attacker
    sub ebx, eax

    ; ebx - lhs attacker - rhs attacker - 8 * (lhs victim - rhs victim)
    shr edx, 6
    xor r8, 48
    call board_get_piece ; rhs victim
    shl eax, 3
    add ebx, eax
    sets al

    xor r8, 48
    pop rbx
    ret
