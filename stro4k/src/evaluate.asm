MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 80
EG_BISHOP_PAIR equ 182

MG_OPEN_FILE equ 83
EG_OPEN_FILE equ 3
MG_SEMI_OPEN_FILE equ 41
EG_SEMI_OPEN_FILE equ 0

section .rodata
EVAL_WEIGHTS:
MATERIAL_EVAL:
    dw 289, 278
    dw 747 - 4 * 30, 647 - 4 * 26
    dw 866 - 6 * 25, 679 - 6 * 13
    dw 1131 - 7 * 22, 1197 - 7 * 4
    dw 2518 - 13 * 16, 2119 - 13 * 0

MOBILITY_EVAL:
    db 30, 26
    db 25, 13
    db 22,  4
    db 16,  0

DOUBLED_PAWN_EVAL:
    db -79, -54
    db -46, -33
    db -64, -26
    db -37, -17
    db -34, -21
    db -43, -39
    db -28, -43
    db -37, -44

ISOLATED_PAWN_EVAL:
    db -15, -23
    db -18, -11
    db -41, -24
    db -63, -37
    db -89, -32
    db -44, -30
    db -37, -31
    db -94, -27

; in reverse order because lzcnt is used
PASSED_PAWN_EVAL:
    db 111, 235
    db 124, 145
    db  53,  81
    db   0,  23
    db   0,   0
    db   0,   0

default rel
section .text

; board - rsi
evaluate:
    push rbx
    push rbp
    lea rbp, [EVAL_WEIGHTS]
    mov r10, rsi
    lea r11, [rsi + Board.black_pieces]

    ; r9 - occ
.side_eval_head:
    mov r9, qword [rsi + Board.white]
    or r9, qword [rsi + Board.black]

    mov ecx, 4
    xor ebx, ebx
.material_eval_head:
    popcnt rax, qword [r10 + 8 * rcx]

    ; SWAR multiplication for MG and EG eval
    ; since it must be positive
    imul eax, dword [rbp + 4 * rcx]
    add ebx, eax

    dec ecx
    jns .material_eval_head

    ; bishop pair
    mov rcx, LIGHT_SQUARES
    mov rax, qword [r10 + 16]
    test rax, rcx
    jz .no_bishop_pair
    not rcx
    test rax, rcx
    jz .no_bishop_pair

    add ebx, MG_BISHOP_PAIR + (EG_BISHOP_PAIR << 16)
.no_bishop_pair:

    ; mobility
    mov edi, 4

    ; rsi - move fns
    lea rsi, qword [move_fns + 6]
.mobility_head:
    ; rcx - piece
    mov rcx, qword [r10 + 8 * rdi]

.mobility_piece_head:
    blsi r8, rcx
    jz .mobility_end_piece
    xor rcx, r8

    call rsi

    ; currently mask is all squares
    ; and rax, mask

    popcnt rax, rax

    ; EG << 16 + MG
    movzx edx, word [rbp + MOBILITY_EVAL - EVAL_WEIGHTS + 2 * rdi - 2]
    shl edx, 8
    xchg dl, dh

    imul eax, edx
    add ebx, eax
    jmp .mobility_piece_head
.mobility_end_piece:
    sub rsi, 2
    dec edi
    jnz .mobility_head

    ; doubled and isolated pawns and open file
    ; r9 - file
    mov r9, 0101010101010101h
    xor ecx, ecx ; loop counter
    xor esi, esi ; mg doubled and isolated pawns
    xor edi, edi ; eg doubled and isolated pawns
.doubled_pawns_head:
    mov r8, qword [r10] ; side pawns
    and r8, r9
    jnz .no_semi_open_file

    ; check if the file is fully open
    mov edx, MG_SEMI_OPEN_FILE + (EG_SEMI_OPEN_FILE << 16)
    mov eax, MG_OPEN_FILE + (EG_OPEN_FILE << 16)
    test r9, qword [r11] ; enemy pawns
    cmovz edx, eax

    ; find number of rooks
    mov rax, qword [r10 + 24] ; side rooks
    and rax, r9
    popcnt rax, rax
    imul eax, edx
    add ebx, eax
.no_semi_open_file:
    ; isolated pawns
    ; rdx - adjacent files
    mov rdx, r9
    mov rax, r9

    shr rax, 1
    and rax, qword [NOT_H_FILE] ; potential opportunity to save bytes here
    and rdx, qword [NOT_H_FILE]
    lea rdx, [rax + 2 * rdx]

    ; rax - number of pawns on file
    popcnt rax, r8

    test rdx, qword [r10]
    jnz .no_isolated_pawns

    movsx edx, byte [rbp + ISOLATED_PAWN_EVAL - EVAL_WEIGHTS + 2 * rcx]
    imul edx, eax
    add esi, edx

    movsx edx, byte [rbp + ISOLATED_PAWN_EVAL - EVAL_WEIGHTS + 2 * rcx + 1]
    imul edx, eax
    add edi, edx

.no_isolated_pawns:

    ; saturating subtraction
    sub al, 1
    adc al, 0

    ; doubled pawns
    movsx edx, byte [rbp + DOUBLED_PAWN_EVAL - EVAL_WEIGHTS + 2 * rcx]
    imul edx, eax
    add esi, edx

    movsx edx, byte [rbp + DOUBLED_PAWN_EVAL - EVAL_WEIGHTS + 2 * rcx + 1]
    imul edx, eax
    add edi, edx

    inc ecx
    shl r9, 1
    jnc .doubled_pawns_head

    ; add up mg and eg
    movzx eax, bx
    shr ebx, 16
    add eax, esi
    add ebx, edi

    ; switch white and black
    xchg r10, r11
    cmp r10, r11

    push rbx ; eg
    push rax ; mg
    mov rsi, r11
    ja .side_eval_head


    ; passed pawns
    mov r8, qword [r10] ; white pawns
    mov r9, qword [r11] ; black pawns
    xor r11d, r11d ; loop counter
.white_passed_pawn_head:
    ; get the black pawn attack spans
    ; The leftmost bit triggers the carry flag so that the shifts
    ; are 8, 16, 32
    mov ecx, 20000008h
    mov rax, r9
.passed_pawn_south_head:
    shrx rdx, rax, rcx
    or rax, rdx
    shl ecx, 1
    jnc .passed_pawn_south_head

    ; attack spans
    mov rcx, rax
    mov rdx, rax

    shr rcx, 7
    and rcx, qword [NOT_A_FILE]
    and rdx, qword [NOT_A_FILE]
    shr rdx, 9

    or rax, rdx
    or rax, rcx

    ; rax - passed pawns
    andn rax, rax, r8

    mov rcx, 0101010101010101h
    xor esi, esi ; mg eval
    xor edi, edi ; eg eval
.passed_pawn_files_head:
    mov rdx, rax
    and rdx, rcx ; passed pawns on file
    jz .no_passed_pawn
    lzcnt rdx, rdx
    shr edx, 3

    movzx ebx, word [rbp + PASSED_PAWN_EVAL - EVAL_WEIGHTS - 2 + 2 * rdx]
    movzx edx, bl
    shr ebx, 8

    add esi, edx
    add edi, ebx
.no_passed_pawn:
    shl rcx, 1
    jnc .passed_pawn_files_head

    ; swap white and black
    xchg r9, r8
    bswap r8
    bswap r9
    dec r11d
    jpo .white_passed_pawn_end

    push rdi ; eg
    push rsi ; mg
    jmp .white_passed_pawn_head
.white_passed_pawn_end:
    ; add up all eval terms
    pop rax
    pop rbx
    sub eax, esi
    sub ebx, edi

    ; black eval
    pop rsi
    pop rdi
    sub eax, esi
    sub ebx, edi
    
    ; white eval
    pop rsi
    pop rdi
    add eax, esi
    add ebx, edi

    ; calculate phase
    mov ecx, 4
.phase_head:
    mov rsi, qword [r10 + 8 * rcx]
    or rsi, qword [r10 + 8 * rcx + 48]
    popcnt rdi, rsi
    push rdi
    
    dec ecx
    jnz .phase_head

    pop rdi
    pop rsi
    pop rcx
    pop rdx
    add edi, esi
    lea ecx, [rcx + 2 * rdx]
    lea ecx, [rdi + 2 * rcx]

    ; mg eval
    imul eax, ecx

    ; eg eval
    mov dl, 24 ; top half is zero from phase calculation
    sub edx, ecx
    imul ebx, edx

    ; divide by 24
    add ebx, eax
    movsx rax, ebx
    imul rax, rax, 2aaaaaabh
    mov rcx, rax
    sar rax, 34
    shr rcx, 63
    add eax, ecx

    ; return side to move relative eval
    test byte [r10 + Board.side_to_move], 1
    jz .white_to_move
    neg eax
.white_to_move:

    pop rbp
    pop rbx
    ret

