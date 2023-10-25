MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 45
EG_BISHOP_PAIR equ 91

section .rodata
MATERIAL_EVAL:
    dw  175,  155
    dw  435,  374
    dw  466,  354
    dw  687,  653
    dw 1535, 1157

MOBILITY_EVAL:
    db   12,    2
    db   11,    5
    db    6,    2
    db    6,   -2

; in reverse order because lzcnt is used
PASSED_PAWN_EVAL:
    db   86,   72
    db   70,   56
    db   24,   44
    db  -14,   32
    db  -29,    9
    db  -21,   -4

; doubled and isolated pawn eval
; first two in each row are isolated mg and eg
; second two are doubled mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db  0,  1, 102, 62,
    db 24, 13,  60, 45,
    db 22, 19,  45, 26,
    db 49, 17,  58, 10,
    db 33, 31,  44, 18,
    db 26, 14,  70, 26,
    db 16, 20,  46, 41,
    db 19,  4,  79, 49,

OPEN_FILE_EVAL:
    db   -2,  -14
    db   -8,    1
    db   49,   -8
    db  -13,   27
    db  -62,   -4

SEMI_OPEN_FILE_EVAL:
    db    0,    6
    db   -8,   25
    db   32,    3
    db    6,   13
    db  -18,   19

; 0-4 pawns, 4 is max possible
PAWN_SHIELD_EVAL:
    db  -66,   37
    db   -7,   -1
    db   36,  -10
    db   72,  -11
    db   69,   -2

EVAL_WEIGHTS:
RANK_PST:
    db    0,    0
    db  -25,    4
    db  -22,   -9
    db    8,  -17
    db   32,  -11
    db   55,   11
    db   86,   72
    db    0,    0


    db  -31,  -36
    db  -10,  -25
    db  -13,   -2
    db    8,   22
    db   33,   29
    db   91,    4
    db   81,   -8
    db  -87,   10


    db   -9,  -17
    db    3,  -14
    db    8,   -5
    db    2,    4
    db    1,   12
    db   46,   -2
    db   -3,    4
    db  -64,   19


    db  -17,  -20
    db  -40,  -12
    db  -32,   -3
    db  -21,   11
    db   17,   14
    db   56,    8
    db   69,   14
    db   93,   -3


    db    3,  -86
    db   11,  -75
    db  -13,   -8
    db  -21,   34
    db  -14,   55
    db   29,   42
    db    4,   56
    db   93,  -24


    db   -3,  -44
    db  -19,  -14
    db  -53,    9
    db  -26,   21
    db   31,   27
    db   90,   32
    db  127,   20
    db  127,  -13


FILE_PST:
    db  -32,    6
    db    4,   19
    db  -24,   12
    db    9,   -9
    db    6,    5
    db   30,   -8
    db   26,   -2
    db  -13,  -20


    db  -17,  -23
    db    1,   -8
    db   -5,    8
    db    8,   13
    db    6,   14
    db    9,   -1
    db   12,    1
    db    1,  -18


    db    5,   -6
    db   10,   -3
    db   -4,   -1
    db   -8,    4
    db   -8,    2
    db   -9,    0
    db   23,   -7
    db   22,  -13


    db  -19,    4
    db  -19,    6
    db    2,    7
    db   14,    2
    db   14,   -7
    db    2,    1
    db    5,   -3
    db    5,  -13


    db    0,  -52
    db   -5,  -16
    db   -5,    4
    db   -6,   16
    db   -7,   25
    db    2,   19
    db   31,   -8
    db   47,  -34


    db   30,  -23
    db   35,   -4
    db   -7,    9
    db  -52,   21
    db    5,    3
    db  -75,   18
    db   20,   -7
    db   12,  -25


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
    mov ebx, 10001000h ; 'king' value which cancels out but avoids underflow
.material_eval_head:
    popcnt rax, qword [r10 + 8 * rcx]

    ; SWAR multiplication for MG and EG eval
    ; since it must be positive
    imul eax, dword [rbp + MATERIAL_EVAL - EVAL_WEIGHTS + 4 * rcx]
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
    movsx edx, word [rbp + MOBILITY_EVAL - EVAL_WEIGHTS + 2 * rdi - 2]
    shl edx, 8
    xchg dl, dh

    imul edx, eax
    add ebx, edx
    jmp .mobility_piece_head
.mobility_end_piece:
    sub rsi, 2
    dec edi
    jnz .mobility_head

    ; doubled and isolated pawns and open file
    ; r9 - file
    mov r9, r12
    xor ecx, ecx ; loop counter
.pawn_eval_head:
    mov r8, qword [r10] ; side pawns
    and r8, r9
    jnz .no_semi_open_file

    lea rdi, [rbp + SEMI_OPEN_FILE_EVAL - EVAL_WEIGHTS - 2]

    ; check if the file is fully open
    test r9, qword [r11]
    jnz .no_fully_open_file
    add rdi, OPEN_FILE_EVAL - SEMI_OPEN_FILE_EVAL
.no_fully_open_file:

    mov esi, 5
.open_file_piece_head:
    ; find number of pieces
    mov rax, qword [r10 + rsi * 8] ; side pieces
    and rax, r9
    popcnt rax, rax

    movsx edx, byte [rdi + rsi * 2]
    imul edx, eax
    add bx, dx ; Avoids affecting eg eval

    movsx edx, byte [rdi + rsi * 2 + 1]
    imul edx, eax
    shl edx, 16
    add ebx, edx

    dec esi
    jnz .open_file_piece_head ; exclude pawns
.no_semi_open_file:
    ; isolated pawns
    ; rax - adjacent files
    andn rax, r12, r9
    shl r9, 1
    andn rdx, r12, r9
    shr rax, 1
    add rax, rdx

    ; rdx - number of pawns on file
    popcnt rdx, r8

    ; load isolated and doubled pawns and SWAR-multiply by rdx
    ; is smaller after compression with xmm5 for some unknown reason
    vpmovzxbw xmm5, qword [rbp + DOUBLED_ISOLATED_PAWN_EVAL - EVAL_WEIGHTS + rcx * 4]
    vmovq rdi, xmm5
    imul rdx, rdi

    test rax, qword [r10]
    jnz .no_isolated_pawns

    ; these subtractions cannot underflow because of the king value
    sub ebx, edx
.no_isolated_pawns:
    sub rdx, rdi
    jc .no_doubled_pawns

    shr rdx, 32
    sub ebx, edx
.no_doubled_pawns:
    inc ecx
    cmp ecx, 8
    jne .pawn_eval_head

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

    mov esi, 5
.pst_piece_head:
    mov r8, qword [r10 + rsi * 8]
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

    lea ecx, [rcx + rsi * 8]
    lea r9d, [rdx + rsi * 8]

    ; file
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

    ; White king
    mov edx, dword [r10 + 40]
.white_eval_head:
    ; Black king
    movbe eax, dword [r10 + 88 + 4]

    ; for white, SF=0 from xor
    ; for black, SF=1 from dec at end of loop
    cmovs edx, eax

    mov ecx, 0707E0E0h
    xor esi, esi ; mg eval
    xor edi, edi ; eg eval

.pawn_shield_head:
    movzx eax, cx
    test eax, edx
    jz .pawn_shield_tail

    ; Get number of pawns
    shl eax, 8
    and eax, r8d
    popcnt eax, eax

    movsx esi, byte [rbp + PAWN_SHIELD_EVAL - EVAL_WEIGHTS + rax * 2]
    movsx edi, byte [rbp + PAWN_SHIELD_EVAL - EVAL_WEIGHTS + rax * 2 + 1]
.pawn_shield_tail:
    shr rcx, 16
    jnz .pawn_shield_head

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
    jmp .white_eval_head
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

