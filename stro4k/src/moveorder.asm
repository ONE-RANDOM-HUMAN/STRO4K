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
    and edx, 0FFFF_0000h
    mov r9d, ebp
.inner_loop_head:
    mov edi, dword [r11 + 4 * r9 - 4]

    ; compare moves
    cmp edx, edi
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

