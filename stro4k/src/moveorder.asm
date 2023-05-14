default rel
section .text

; moves - r11
; move count - r12
; less function - r15
; preserves rbx, rbp, r8, r11, r12, r13, r14, r15
sort_moves_flags:
    push rbx
    push rbp

    ; ebp - inner loop counter
    mov ebp, 1
.outer_loop_head:
    cmp ebp, r12d
    jae .end

    movzx r10d, word [r11 + 2 * rbp]
    mov edx, r10d
    and edx, 0F000h
    mov r9d, ebp
.inner_loop_head:
    movzx edi, word [r11 + 2 * r9 - 2]

    ; compare moves
    cmp edx, edi
    jna .inner_loop_end

    mov word [r11 + 2 * r9], di
    dec r9d
    jnz .inner_loop_head
.inner_loop_end:
    mov word [r11 + 2 * r9], r10w
    inc ebp
    jmp .outer_loop_head
.end:
    pop rbp
    pop rbx
    ret


; board - rsi
; moves - r11
; move count - r12
; less function - r15
; preserves rbx, rbp, r8, r11, r12, r13, r14, r15
sort_moves_mvvlva:
    push rbx
    push rbp

    ; ebp - inner loop counter
    mov ebp, 1
.outer_loop_head:
    cmp ebp, r12d
    jae .end

    movzx r10d, word [r11 + 2 * rbp]
    mov r9d, ebp

    ; attacker
    mov edx, r10d
    call board_get_piece
    mov ebx, eax
    shr edx, 6

    ; victim
    xor r8, 48 ; switch pieces, taking advantage of 128 byte alignment
    call board_get_piece
    shl eax, 3 
    sub ebx, eax

    xor r8, 48

.inner_loop_head:
    movzx edi, word [r11 + 2 * r9 - 2]

    ; attacker
    mov edx, edi
    call board_get_piece ; rhs attacker
    mov esi, eax
    shr edx, 6

    ; victim
    xor r8, 48
    call board_get_piece ; rhs victim
    shl eax, 3
    sub esi, eax

    xor r8, 48

    ; compare moves
    cmp ebx, esi
    jnl .inner_loop_end

    mov word [r11 + 2 * r9], di
    dec r9d
    jnz .inner_loop_head
.inner_loop_end:
    mov word [r11 + 2 * r9], r10w
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

    movzx r10d, word [r11 + 2 * rbp]
    mov edx, r10d
    and edx, 0FFFh
    mov rdx, qword [r8 + 8 * rdx]

    mov r9d, ebp
.inner_loop_head:
    movzx edi, word [r11 + 2 * r9 - 2]

    mov esi, edi
    and esi, 0FFFh

    ; compare moves
    cmp rdx, qword [r8 + 8 * rsi]
    jng .inner_loop_end

    mov word [r11 + 2 * r9], di
    dec r9d
    jnz .inner_loop_head
.inner_loop_end:
    mov word [r11 + 2 * r9], r10w
    inc ebp
    jmp .outer_loop_head
.end:
    pop rbp
    pop rbx
    ret

