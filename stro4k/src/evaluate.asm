MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 109
EG_BISHOP_PAIR equ 160

MG_OPEN_FILE equ 70
EG_OPEN_FILE equ 0
MG_SEMI_OPEN_FILE equ 37
EG_SEMI_OPEN_FILE equ 1

section .rodata
EVAL_WEIGHTS:
MATERIAL_EVAL:
    dw 325, 309
    dw 824 - 4 * 36, 754 - 4 * 19
    dw 906 - 6 * 24, 789 - 6 * 10
    dw 1275 - 7 * 14, 1342 - 7 * 3
    dw 2594 - 13 * 11, 2451 - 13 * 2

MOBILITY_EVAL:
    db 36, 19
    db 24, 10
    db 14,  3
    db 11,  2

; in reverse order because lzcnt is used
PASSED_PAWN_EVAL:
    db  80, 161
    db  80,  88
    db  37,  73
    db   0,  47
    db   0,   0
    db   0,   0

; doubled and isolated pawn eval
; first two in each row are isolated mg and eg
; second two are doubled mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db 29, 11, 52, 51
    db 27, 21, 39, 38
    db 46, 32, 45, 29
    db 81, 34, 41, 13
    db 71, 48, 53, 23
    db 45, 38, 76, 47
    db 33, 33, 38, 40
    db 76, 28, 38, 44

FILE_PST:
    db -44,  25
    db -25,  37
    db -31,  22
    db   4,   2
    db  20,   6
    db  59,   2
    db  43, -13
    db   3, -17
               
               
    db   2,  -2
    db   3,  -3
    db -20,  -8
    db   1,  -1
    db  -5,  -0
    db  -6, -17
    db  20,   9
    db  22,  -2
               
               
    db  13,  -2
    db  21,  -1
    db  -6,  -1
    db -16,   4
    db -17,   4
    db -13,  -1
    db  34,  -4
    db  30, -10
               
               
    db -35,  10
    db -28,  12
    db  11,  12
    db  35,   3
    db  31, -10
    db   3,  -2
    db  -5,   2
    db -32,  -3
               
               
    db -17, -28
    db -19, -10
    db  -1,  -2
    db  -4,   5
    db -11,  12
    db  -1,  10
    db  24,   4
    db  40,   3
               
               
    db  14, -17
    db  56,   6
    db  18,  11
    db -62,  13
    db -45,  12
    db -79,  20
    db  62, -11
    db  24, -34
    

RANK_PST:
    db   0,   0
    db -44,  -9
    db -49, -34
    db -10, -43
    db  35, -21
    db  69,  35
    db  72,  91
    db   0,   0
               
               
    db -34, -33
    db -28, -37
    db -49, -28
    db  -6,  15
    db  39,  29
    db  68,  10
    db  38,  15
    db  -7,   3
               
               
    db -13, -24
    db   0, -19
    db   9,  -3
    db  -3,   9
    db  11,  15
    db  49,   6
    db  -0,   0
    db  -6,   4
               
               
    db -27, -45
    db -71, -32
    db -47, -20
    db -27,   4
    db  19,  23
    db  43,  26
    db  57,  44
    db  39,  23
               
               
    db -17, -51
    db  -9, -59
    db -29, -14
    db -27,  13
    db  -2,  27
    db  42,  32
    db  22,  34
    db  31,  10
               
               
    db  11, -80
    db -32, -28
    db -36, -11
    db   2,  15
    db  17,  39
    db  19,  52
    db  15,  36
    db   8,   9
    

default rel
section .text

; board - rsi
evaluate:
    push rbx
    push rbp
    lea rbp, [EVAL_WEIGHTS]
    mov r10, rsi
    lea r11, [rsi + Board.black_pieces]

    mov r12, 0101010101010101h

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
    mov r9, r12
    xor ecx, ecx ; loop counter
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
    ; rax - adjacent files
    lea rdx, [r9 + r9]
    andn rdx, r12, rdx
    andn rax, r12, r9
    shr rax, 1
    add rax, rdx

    ; rdx - number of pawns on file
    popcnt rdx, r8

    ; load isolated and doubled pawns and SWAR-multiply by rdx
    vpmovzxbw xmm0, qword [rbp + DOUBLED_ISOLATED_PAWN_EVAL - EVAL_WEIGHTS + rcx * 4]
    vmovq rdi, xmm0
    mulx rdx, rsi, rdi ; rdx is implicit source

    test rax, qword [r10]
    jnz .no_isolated_pawns

    ; these subtractions cannot overlow because the penalty for doubled
    ; and isolated pawns is less than the value of a pawn
    sub ebx, esi
.no_isolated_pawns:
    sub rsi, rdi
    jc .no_doubled_pawns

    shr rsi, 32
    sub ebx, esi
.no_doubled_pawns:
    inc ecx
    shl r9, 1
    jnc .doubled_pawns_head

    ; add up mg and eg
    movzx eax, bx
    shr ebx, 16

    ; pst eval
    ; ebx - eg
    ; eax - mg

    ; side to move
    cmp r11, r10 ; sets CF if r11 < r10
    sbb edi, edi ; -1 if black pieces
    and edi, 111b

    mov esi, 40
.pst_piece_head:
    mov r8, qword [r10 + rsi]
.pst_square_head:
    tzcnt rcx, r8
    jc .pst_tail
    btr r8, rcx

    ; ecx - file index
    ; edx - rank index
    mov edx, ecx
    shr edx, 3
    and ecx, 111b
    xor edx, edi

    lea ecx, [rcx + rsi]
    lea r9d, [rdx + rsi]

    ; probably some byte savings here by merging rank/file psts
    movzx ecx, word [rbp + FILE_PST - EVAL_WEIGHTS + 2 * rcx]
    movsx edx, cl
    add eax, edx
    movsx edx, ch
    add ebx, edx

    ; rank
    movzx ecx, word [rbp + RANK_PST - EVAL_WEIGHTS + 2 * r9]
    movsx edx, cl
    add eax, edx
    movsx edx, ch
    add ebx, edx

    jmp .pst_square_head
.pst_tail:
    sub esi, 8
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

    shr rcx, 7
    andn rcx, r12, rcx
    andn rdx, r12, rax
    shr rdx, 9

    or rax, rdx
    or rax, rcx

    ; rax - passed pawns
    andn rax, rax, r8

    mov rcx, r12
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

