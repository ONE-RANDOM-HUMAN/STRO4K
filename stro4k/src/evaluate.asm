MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 90
EG_BISHOP_PAIR equ 163

MG_OPEN_FILE equ 68
EG_OPEN_FILE equ 0
MG_SEMI_OPEN_FILE equ 30
EG_SEMI_OPEN_FILE equ 0

section .rodata
EVAL_WEIGHTS:
MATERIAL_EVAL:
    dw 331, 297
    dw 788 - 4 * 33, 693 - 4 * 19
    dw 885 - 6 * 21, 709 - 6 * 10
    dw 1210 - 7 * 16, 1251 - 7 * 2
    dw 2484 - 13 * 11, 2260 - 13 * 0

MOBILITY_EVAL:
    db 33, 19
    db 21, 10
    db 16,  2
    db 11,  0

DOUBLED_PAWN_EVAL:
    db -84,  16
    db -39,  24
    db -45,  11
    db -25,  -7
    db -10,   2
    db  24, -14
    db  24,  -2
    db -38, -39

; in reverse order because lzcnt is used
PASSED_PAWN_EVAL:
    db 80, 141
    db 91, 104
    db 17,  46
    db  0,  28
    db  0,   0
    db  0,   0

PST_MG:
    db  -7
    db -54
    db -14
    db -33
    db  30
    db  41
    db  38
    db  41
    db -32
    db -38
    db -26
    db -46
    db  61
    db  35
    db  11
    db  36
    db  20
    db -39
    db   7
    db -14
    db   9
    db  36
    db  -8
    db  -2
    db -69
    db -16
    db -41
    db -32
    db  22
    db  35
    db  41
    db  55
    db -19
    db  -7
    db -19
    db -31
    db  27
    db  11
    db   9
    db  29
    db  47
    db -58
    db -16
    db -21
    db  14
    db  21
    db  10
    db  12

PST_EG:
    db -20
    db  10
    db -31
    db -41
    db   4
    db -30
    db  75
    db  59
    db -22
    db -40
    db  -2
    db -14
    db  23
    db  14
    db   2
    db  13
    db -19
    db -23
    db  -3
    db   3
    db  16
    db   5
    db   2
    db   4
    db -43
    db -48
    db -18
    db -10
    db  11
    db  21
    db  27
    db  33
    db -33
    db -75
    db  -2
    db   8
    db   8
    db  44
    db  -3
    db  33
    db -62
    db -39
    db -16
    db   7
    db  26
    db  47
    db   9
    db  26


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

    ; doubled pawns and open file
    ; r9 - file
    mov r9, 0101010101010101h
    xor ecx, ecx ; loop counter
    xor esi, esi ; mg doubled pawns
    xor edi, edi ; eg doubled pawns
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
    popcnt rax, r8

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

    ; pst eval
    ; ebx - eg
    ; eax - mg

    ; side to move
    cmp r11, r10 ; sets CF if r11 < r10
    sbb edi, edi ; -1 if black pieces
    and edi, 11b

    mov esi, 5
.pst_piece_head:
    mov r8, qword [r10 + 8 * rsi]
.pst_square_head:
    xor edx, edx
    tzcnt rcx, r8
    jc .pst_tail
    btr r8, rcx

    ; dl - column
    test cl, 110b
    setpo dl

    ; cl - row
    shr ecx, 4
    xor ecx, edi ; flip vertically for black pieces

    ; ecx - index
    lea ecx, [rdx + 2 * rcx]
    lea ecx, [rcx + 8 * rsi]

    movsx edx, byte [rbp + PST_MG - EVAL_WEIGHTS + rcx]
    add eax, edx

    movsx edx, byte [rbp + PST_EG - EVAL_WEIGHTS + rcx]
    add ebx, edx
    jmp .pst_square_head
.pst_tail:
    dec esi
    jns .pst_piece_head

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

