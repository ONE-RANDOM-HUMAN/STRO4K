default rel
section .text

; moves - r11
; move count - r12
; preserves rbx, rbp, r8, r11, r12, r13, r14, r15
sort_moves_by_score:
    push rbx
    push rbp

    ; ebp - inner loop counter
    mov ebp, 1
.outer_loop_head:
    cmp ebp, r12d
    jae .end

    mov r10d, dword [r11 + 4 * rbp]
    mov edx, r10d
    and edx, 0FFF_0000h
    mov r9d, ebp
.inner_loop_head:
    mov edi, dword [r11 + 4 * r9 - 4]

    ; compare moves
    cmp edx, edi
    jna .inner_loop_end

    mov dword [r11 + 4 * r9], edi
    dec r9d
    jnz .inner_loop_head
.inner_loop_end:
    mov dword [r11 + 4 * r9], r10d
    inc ebp
    jmp .outer_loop_head
.end:
    pop rbp
    pop rbx
    ret

; moves - r11
; move count - r12
; history - r8
; preserves rbx, rbp, r8, r11, r12, r13, r14, r15
sort_moves_history:
    push rbx
    push rbp

    ; ebp - inner loop counter
    mov ebp, 1
.outer_loop_head:
    cmp ebp, r12d
    jae .end

    mov r10d, dword [r11 + 4 * rbp]
    mov edx, r10d
    and edx, 0FFFh
    mov rdx, qword [r8 + 8 * rdx]

    mov r9d, ebp
.inner_loop_head:
    mov edi, dword [r11 + 4 * r9 - 4]

    mov esi, edi
    and esi, 0FFFh

    ; compare moves
    cmp rdx, qword [r8 + 8 * rsi]
    jng .inner_loop_end

    mov dword [r11 + 4 * r9], edi
    dec r9d
    jnz .inner_loop_head
.inner_loop_end:
    mov dword [r11 + 4 * r9], r10d
    inc ebp
    jmp .outer_loop_head
.end:
    pop rbp
    pop rbx
    ret

