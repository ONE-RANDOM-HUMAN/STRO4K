MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 60
EG_BISHOP_PAIR equ 125

MG_TEMPO equ 90
EG_TEMPO equ 47

section .rodata
MATERIAL_EVAL:
    dw  174,  244
    dw  522,  571
    dw  519,  547
    dw  861, 1045
    dw 1954, 2207

; For smaller size
BISHOP_PAIR_EVAL:
    dw MG_BISHOP_PAIR, EG_BISHOP_PAIR

TEMPO_EVAL:
    db MG_TEMPO, EG_TEMPO

MOBILITY_EVAL:
    db    5,   -2
    db   11,    5
    db    4,    2
    db    7,  -33

PASSED_PAWN_EVAL:
    db   -6,  -24
    db  -27,   -1
    db  -27,   38
    db   23,   57
    db   72,   72
    db   76,  115


; first two in each row are doubled mg and eg
; second two are isolated mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db  -95, -101,    5,    8
    db  -57,  -74,  -22,  -14
    db  -42,  -42,  -15,  -21
    db  -68,  -23,  -41,  -25
    db  -27,  -35,  -42,  -33
    db  -51,  -53,  -34,  -10
    db  -25,  -76,  -23,  -17
    db  -55,  -92,  -40,    2


OPEN_FILE_EVAL:
    db   -8,  -13
    db  -17,    2
    db   53,   -1
    db  -11,   17
    db  -72,  -21

SEMI_OPEN_FILE_EVAL:
    db   -6,    9
    db  -17,   29
    db   28,    5
    db    3,  -11
    db  -26,   24

; 0-4 pawns, 4 is max possible
PAWN_SHIELD_EVAL:
    db  -71,   36
    db  -21,    0
    db   34,   -9
    db   81,   -5
    db   73,   20

EVAL_WEIGHTS:
PAWN_DEFENDED_EVAL:
    db   27,   16
    db    6,   18
    db   -1,   27
    db   12,   22
    db  -10,   25
    db -103,   68

PAWN_ATTACKED_EVAL:
    db   28,   43
    db  -94,  -96
    db  -86, -112
    db  -83, -108
    db  -83,  -73
    db -102,   24

RANK_PST:
    db    0,    0
    db  -27,  -10
    db  -38,  -32
    db   10,  -40
    db   46,  -25
    db   77,   15
    db   76,  115
    db    0,    0


    db  -96,  -67
    db  -45,  -46
    db  -24,   -5
    db   10,   28
    db   45,   35
    db  122,    8
    db   75,  -21
    db -109,  -32


    db  -47,  -37
    db  -18,  -28
    db   -7,  -16
    db   -3,   -6
    db    5,    0
    db   55,  -16
    db   -6,  -10
    db -120,    1


    db  -98,  -75
    db -120,  -66
    db -110,  -60
    db  -97,  -31
    db  -51,  -21
    db    2,  -25
    db   10,  -19
    db   -1,  -34


    db -128, -128
    db -108, -128
    db -112,  -84
    db -107,  -18
    db  -80,  -29
    db  -25,  -92
    db  -56, -105
    db  -69, -128


    db   32,  -87
    db    6,  -34
    db  -43,    1
    db  -49,   35
    db  -60,   78
    db  -36,   98
    db   55,   51
    db   70,  -25


FILE_PST:
    db  -34,    7
    db    4,   29
    db  -28,   10
    db    5,  -11
    db    5,   -1
    db   28,  -12
    db   22,   -2
    db  -12,  -29


    db  -72,  -58
    db  -23,  -32
    db  -17,    4
    db    0,    8
    db   -5,    7
    db    1,  -13
    db  -12,  -20
    db  -45,  -53


    db   -9,  -32
    db    0,  -21
    db  -25,  -14
    db  -29,   -5
    db  -27,  -11
    db  -31,  -13
    db    5,  -26
    db    8,  -37


    db  -98,  -48
    db  -97,  -43
    db  -65,  -46
    db  -47,  -50
    db  -53,  -64
    db  -81,  -54
    db  -77,  -57
    db  -72,  -68


    db -124, -128
    db -108, -128
    db  -98,  -89
    db -101,  -63
    db -104,  -61
    db  -91, -110
    db  -68, -128
    db  -77, -128


    db   76,  -57
    db   65,  -11
    db    3,   16
    db  -88,   37
    db  -16,   17
    db  -96,   31
    db   39,   -9
    db   28,  -45


default rel
section .text

; board - rsi
evaluate:
    push rbx
    push rbp
    push r13
    push r14
    lea rbp, [EVAL_WEIGHTS]

    ; Side to move
    mov r10, rsi
    lea r11, [rsi + Board.black_pieces]

    ; Pawn attacks
    mov r8, 0101010101010101h

    ; white
    mov rdi, qword [r10]
    andn rax, r8, rdi
    shl rax, 7
    shl rdi, 9
    andn r13, r8, rdi
    or r13, rax

    ; black
    mov rdi, qword [r11]
    andn rax, r8, rdi
    shr rax, 9
    shr rdi, 7
    andn r14, r8, rdi
    or r14, rax

    cmp byte [rsi + Board.side_to_move], 0
    je .white_to_move
    xchg r10, r11
    xchg r13, r14
.white_to_move:
    vpmovsxbw xmm0, qword [rbp + TEMPO_EVAL - EVAL_WEIGHTS]

    ; r9 - occ
    mov r9, qword [rsi + Board.white]
    or r9, qword [rsi + Board.black]
.side_eval_head:

    ; bishop pair
    mov rbx, LIGHT_SQUARES
    mov rax, qword [r10 + 16]
    test rax, rbx
    jz .no_bishop_pair
    andn rax, rbx, rax
    jz .no_bishop_pair

    vpaddw xmm0, xmm0, oword [rbp + BISHOP_PAIR_EVAL - EVAL_WEIGHTS]
.no_bishop_pair:

    ; Pawn shield
    mov rax, qword [r10] ; pawns
    mov rdx, qword [r10 + 40] ; king

    cmp r11, r10
    ja .pawn_shield_white

    bswap rax
    bswap rdx
.pawn_shield_white:
    mov ecx, 0707h
    test edx, ecx
    jnz .pawn_shield

    mov ecx, 0E0E0h
    test edx, ecx
    jz .no_pawn_shield
.pawn_shield:
    shl ecx, 8
    and eax, ecx
    popcnt eax, eax

    vpmovsxbw xmm1, qword [rbp + PAWN_SHIELD_EVAL - EVAL_WEIGHTS + 2 * rax]
    vpaddw xmm0, xmm0, xmm1
.no_pawn_shield:

    mov ecx, 5
.side_pieces_head:
    mov r12, qword [r10 + 8 * rcx]
.piece_type_head:
    ; ebx - piece index
    tzcnt rbx, r12
    jc .piece_type_end
    btr r12, rbx

    ; Material
    ; It does not matter if a random value is added for
    ; king eval because it cancels out anyway
    vpaddw xmm0, xmm0, oword [rbp + MATERIAL_EVAL - EVAL_WEIGHTS + 4 * rcx]

    ; pawn defended
    bt r13, rbx
    jnc .not_pawn_defended

    vpmovsxbw xmm1, qword [rbp + PAWN_DEFENDED_EVAL - EVAL_WEIGHTS + 2 * rcx]
    vpaddw xmm0, xmm0, xmm1
.not_pawn_defended:

    ; pawn attacked
    bt r14, rbx
    jnc .not_pawn_attacked

    vpmovsxbw xmm1, qword [rbp + PAWN_ATTACKED_EVAL - EVAL_WEIGHTS + 2 * rcx]
    vpaddw xmm0, xmm0, xmm1
.not_pawn_attacked:

    ; mobility
    cmp ecx, 0
    je .no_mobility
    cmp ecx, 5
    je .no_mobility

    ; rax - move fn
    lea rax, [move_fns - 2]
    lea rax, [rax + 2 * rcx]

    ; r8 - piece
    ; r9 - occ
    xor r8d, r8d
    bts r8, rbx

    call rax

    ; currently mask is all squares
    ; and rax, mask

    popcnt rax, rax

    vmovd xmm2, eax
    vpmovsxbw xmm1, qword [rbp + MOBILITY_EVAL - EVAL_WEIGHTS + 2 * rcx - 2]
    vpmulld xmm1, xmm1, xmm2

    ; Alternatively with multiplication in GPRs
    ; EG << 16 + MG
    ; movsx edx, word [rbp + MOBILITY_EVAL - EVAL_WEIGHTS + 2 * rcx - 2]
    ; shl edx, 8
    ; xchg dl, dh
    ;
    ; imul edx, eax
    ; vmovd xmm1, edx
    vpaddw xmm0, xmm0, xmm1
.no_mobility:
    cmp r11, r10 ; sets CF if r11 < r10
    sbb eax, eax ; -1 if black pieces

    ; edx - file index
    ; eax - rank index
    mov edx, ebx
    xor eax, ebx
    shr eax, 3
    and edx, 111b
    and eax, 111b

    ; file
    lea edi, [rdx + rcx * 8]
    vpmovsxbw xmm1, qword [rbp + FILE_PST - EVAL_WEIGHTS + 2 * rdi]
    vpaddw xmm0, xmm0, xmm1

    ; rank
    lea edi, [rax + rcx * 8]
    vpmovsxbw xmm1, qword [rbp + RANK_PST - EVAL_WEIGHTS + 2 * rdi]
    vpaddw xmm0, xmm0, xmm1

    ; r8 - A-file
    mov r8, 0101010101010101h

    ; Free up rdx and rax
    vpmovsxbw xmm1, qword [rbp + DOUBLED_ISOLATED_PAWN_EVAL - EVAL_WEIGHTS + 4 * rdx]
    vpmovsxbw xmm2, qword [rbp + PASSED_PAWN_EVAL - EVAL_WEIGHTS + 2 * rax - 2]

    test ecx, ecx
    jnz .not_pawn_eval

    ; Doubled, isolated, and passed pawns

    ; rdi - mask in front of pawn
    ; rdx - file mask
    shlx rdi, r8, rbx
    shlx rdx, r8, rdx
    cmp r11, r10
    ja .pawn_eval_white_piece

    xor rdi, rdx
.pawn_eval_white_piece:
    btr rdi, rbx
    test rdi, qword [r10]
    jz .no_doubled_pawn

    vpaddw xmm0, xmm0, xmm1
    jmp .no_passed_pawn
.no_doubled_pawn:
    ; passed pawn
    bts rdi, rbx
    test rdi, qword [r11]
    jnz .no_passed_pawn

    test rdi, r14
    jnz .no_passed_pawn

    vpaddw xmm0, xmm0, xmm2
.no_passed_pawn:
    ; isolated pawn
    test rdx, r13
    jnz .no_isolated_pawn

    vpshufd xmm1, xmm1, 01h
    vpaddw xmm0, xmm0, xmm1
.no_isolated_pawn:
    jmp .not_piece_eval
.not_pawn_eval:
    ; Non-pawn eval
    ; Open files

    ; edx - file index
    shlx rdi, r8, rdx
    test rdi, qword [r10]
    jnz .closed_file

    test rdi, qword [r11]
    jnz .semi_open_file

    vpmovsxbw xmm1, qword [rbp + OPEN_FILE_EVAL - EVAL_WEIGHTS + 2 * rcx - 2]
    vpaddw xmm0, xmm0, xmm1
    jmp .open_file_end
.semi_open_file:
    vpmovsxbw xmm1, qword [rbp + SEMI_OPEN_FILE_EVAL - EVAL_WEIGHTS + 2 * rcx - 2]
    vpaddw xmm0, xmm0, xmm1

.closed_file:
.open_file_end:
.not_piece_eval:

    jmp .piece_type_head
.piece_type_end:

    dec ecx
    jns .side_pieces_head

    vpxor xmm1, xmm1, xmm1
    vpsubw xmm0, xmm1, xmm0

    xchg r10, r11
    xchg r13, r14

    ; Since rsi is a pointer to a board, it must be aligned
    ; so we can loop twice by testing and complementing the
    ; least significant bit
    btc rsi, 0
    jnc .side_eval_head

    mov ecx, 4
    xor eax, eax
.phase_head:
    mov rdi, qword [r10 + 8 * rcx]
    or rdi, qword [r11 + 8 * rcx]
    popcnt rdi, rdi
    lea eax, [rdi + 2 * rax]
    dec ecx
    jnz .phase_head

    ; Add knight eval for 2 * (N + B + 2 * R + 4 * Q)
    ; eax - 2 * phase
    add eax, edi

    vmovd ecx, xmm0
    movsx ebx, cx
    sar ecx, 16

    imul ebx, eax

    sub eax, 48 ; 2 * -(24 - phase)
    imul ecx, eax

    sub ebx, ecx
    movsx rax, ebx

    ; divide by 2 * 24
    imul rax, rax, 2aaaaaabh
    mov rcx, rax
    sar rax, 35
    shr rcx, 63
    add eax, ecx

    pop r14
    pop r13
    pop rbp
    pop rbx
    ret

