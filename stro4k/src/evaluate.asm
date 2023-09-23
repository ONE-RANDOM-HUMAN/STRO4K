MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 92
EG_BISHOP_PAIR equ 176

MG_OPEN_FILE equ 72
EG_OPEN_FILE equ -5
MG_SEMI_OPEN_FILE equ 44
EG_SEMI_OPEN_FILE equ -4

section .rodata
EVAL_WEIGHTS:
MATERIAL_EVAL:
    dw  317,  334
    dw  717,  630
    dw  814,  688
    dw 1130, 1298
    dw 2609, 2298

MOBILITY_EVAL:
    db 31, 22
    db 22, 10
    db 19,  1
    db 13,  0

; in reverse order because lzcnt is used
PASSED_PAWN_EVAL:
    db 126, 122
    db 110,  89
    db  40,  32
    db -23,  25
    db -58, -15
    db -30, -44

; doubled and isolated pawn eval
; first two in each row are isolated mg and eg
; second two are doubled mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db 17,  9, 125, 117
    db 13,  5,  67,  78
    db 50, 22,  96,  54
    db 62, 36,  84,  47
    db 88, 35,  84,  44
    db 45, 31,  64,  70
    db 40, 13,  24,  71
    db 76, 24,  98, 104

PST_MG:
    db  -26
    db  -56
    db   -7
    db   -1
    db  -34
    db   -6
    db   -4
    db   -8
    db   12
    db   36
    db   52
    db   33
    db   54
    db   54
    db   48
    db   21


    db  -21
    db  -30
    db  -19
    db  -21
    db  -35
    db  -25
    db  -29
    db    3
    db   43
    db   46
    db   28
    db   56
    db    0
    db   52
    db   56
    db   23


    db   24
    db  -25
    db  -26
    db   34
    db   11
    db   10
    db  -15
    db   11
    db    1
    db   36
    db   30
    db   21
    db  -29
    db   -9
    db    3
    db   -4


    db  -35
    db   18
    db   10
    db  -37
    db  -40
    db  -15
    db  -13
    db   12
    db   26
    db   41
    db   48
    db   55
    db   48
    db   56
    db   57
    db   59


    db  -21
    db    1
    db  -13
    db  -31
    db  -37
    db  -35
    db  -25
    db   18
    db  -30
    db  -20
    db   38
    db   53
    db  -33
    db   24
    db   52
    db   59


    db   52
    db  -14
    db  -47
    db   42
    db   18
    db   -5
    db  -28
    db  -30
    db   51
    db   47
    db   47
    db   41
    db   54
    db   54
    db   55
    db   51


PST_EG:
    db   -7
    db   18
    db    6
    db  -37
    db  -18
    db  -38
    db  -30
    db  -49
    db   24
    db   -7
    db  -17
    db  -17
    db   52
    db   51
    db   48
    db   42


    db  -28
    db  -33
    db  -32
    db  -13
    db    6
    db   -8
    db  -17
    db   13
    db   31
    db   18
    db   29
    db   50
    db   21
    db   27
    db   25
    db    3


    db  -20
    db  -24
    db  -14
    db  -23
    db    7
    db    7
    db    5
    db   -7
    db   19
    db    9
    db   16
    db   30
    db   15
    db   19
    db   14
    db    4


    db  -32
    db  -37
    db  -43
    db  -38
    db    4
    db    7
    db   -2
    db  -15
    db   30
    db   36
    db   31
    db   14
    db   41
    db   49
    db   42
    db   36


    db  -51
    db  -57
    db  -59
    db  -56
    db  -15
    db   16
    db    5
    db   -2
    db   -1
    db   53
    db   55
    db   50
    db   -3
    db   47
    db   50
    db   34


    db  -28
    db  -28
    db  -30
    db  -44
    db   -2
    db   12
    db   12
    db   -8
    db   36
    db   40
    db   41
    db   30
    db   22
    db   41
    db   47
    db   32


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
    lea rsi, [move_fns + 6]
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
    and edi, 1100b

    mov esi, 40
.pst_piece_head:
    mov r8, qword [r10 + rsi]
.pst_square_head:
    xor edx, edx
    tzcnt rcx, r8
    jc .pst_tail
    btr r8, rcx

    ; ecx - index
    mov edx, 110110b
    pext ecx, ecx, edx
    xor ecx, edi

    lea ecx, [rcx + 2 * rsi]

    movsx edx, byte [rbp + PST_MG - EVAL_WEIGHTS + rcx]
    add eax, edx

    movsx edx, byte [rbp + PST_EG - EVAL_WEIGHTS + rcx]
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

    movsx ebx, word [rbp + PASSED_PAWN_EVAL - EVAL_WEIGHTS - 2 + 2 * rdx]
    movsx edx, bl
    sar ebx, 8

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

