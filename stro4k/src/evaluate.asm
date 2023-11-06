MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 44
EG_BISHOP_PAIR equ 90

MG_TEMPO equ 34
EG_TEMPO equ 4

section .rodata
MATERIAL_EVAL:
    dw  173,  160
    dw  438,  355
    dw  467,  351
    dw  678,  656
    dw 1538, 1121

; For smaller size
BISHOP_PAIR_EVAL:
    dw MG_BISHOP_PAIR, EG_BISHOP_PAIR

TEMPO_EVAL:
    db MG_TEMPO, EG_TEMPO

MOBILITY_EVAL:
    db   12,    6
    db   12,    5
    db    7,    1
    db    6,   -1

PASSED_PAWN_EVAL:
    db  -12,  -10
    db  -23,    2
    db  -10,   25
    db   27,   35
    db   63,   42
    db   73,   66


; first two in each row are doubled mg and eg
; second two are isolated mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db  -71,  -61,   -4,   -1
    db  -45,  -45,  -20,  -10
    db  -38,  -27,  -22,  -17
    db  -47,  -17,  -47,  -16
    db  -36,  -22,  -36,  -25
    db  -53,  -31,  -26,  -14
    db  -33,  -40,  -18,  -16
    db  -55,  -52,  -27,   -5


OPEN_FILE_EVAL:
    db   -3,  -12
    db  -10,    0
    db   44,   -4
    db  -13,   20
    db  -58,   -5

SEMI_OPEN_FILE_EVAL:
    db   -1,    6
    db   -7,   22
    db   27,    4
    db    5,    8
    db  -19,   16

; 0-4 pawns, 4 is max possible
PAWN_SHIELD_EVAL:
    db  -56,   22
    db  -19,   -6
    db   19,  -13
    db   53,  -11
    db   32,    6

EVAL_WEIGHTS:
RANK_PST:
    db    0,    0
    db  -28,    3
    db  -22,  -12
    db    6,  -18
    db   28,   -9
    db   51,   16
    db   73,   66
    db    0,    0


    db  -38,  -30
    db  -15,  -24
    db  -18,   -7
    db    4,   16
    db   28,   22
    db   70,    6
    db   61,    1
    db  -47,    6


    db  -15,  -15
    db    0,  -12
    db    6,   -2
    db   -1,    5
    db    1,   13
    db   41,    2
    db   -2,    4
    db  -36,   12


    db  -19,  -24
    db  -39,  -15
    db  -30,   -8
    db  -20,    6
    db   15,   12
    db   46,    8
    db   57,   16
    db   62,    4


    db   -3,  -61
    db    5,  -60
    db  -15,   -7
    db  -20,   30
    db   -8,   45
    db   29,   37
    db   11,   47
    db   54,    3


    db    3,  -39
    db  -14,  -11
    db  -43,    5
    db  -11,   15
    db   26,   22
    db   55,   32
    db   66,   27
    db   62,    5


FILE_PST:
    db  -27,    5
    db   -4,   18
    db  -21,   10
    db    8,   -7
    db    7,    2
    db   27,   -7
    db   21,   -4
    db  -10,  -19


    db  -19,  -15
    db   -3,   -5
    db   -6,    3
    db    6,    9
    db    4,    6
    db    5,   -5
    db    8,    3
    db   -2,   -9


    db    5,   -4
    db    8,    0
    db   -6,    1
    db   -9,    5
    db   -9,    3
    db  -12,    3
    db   18,   -5
    db   20,  -11


    db  -20,    4
    db  -15,    5
    db    5,    4
    db   16,    0
    db   16,   -7
    db    0,    1
    db    2,   -2
    db   -4,  -11


    db   -5,  -39
    db   -8,  -13
    db   -5,    2
    db   -7,   14
    db   -8,   19
    db   -1,   17
    db   23,   -2
    db   38,  -16


    db   26,  -18
    db   38,   -1
    db    1,    8
    db  -48,   17
    db   -6,    2
    db  -60,   15
    db   25,   -6
    db   14,  -22


default rel
section .text

; board - rsi
evaluate:
    push rbx
    push rbp
    lea rbp, [EVAL_WEIGHTS]

    ; Side to move
    mov r10, rsi
    lea r11, [rsi + Board.black_pieces]
    cmp byte [rsi + Board.side_to_move], 0
    je .white_to_move
    xchg r10, r11
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
    ; passed pawn - might be possible to merge with isolated pawn eval
    andn rax, r8, rdi
    shr rax, 1
    or rax, rdi

    and rdi, qword [NOT_H_FILE] ; Enables the use of Lea instructions
    lea rdi, [rax + 2 * rdi]
    test rdi, qword [r11]
    jnz .no_passed_pawn

    vpaddw xmm0, xmm0, xmm2
.no_passed_pawn:
    ; isolated pawn
    mov rdi, rdx

    andn rax, r8, rdi
    shr rax, 1
    and rdi, qword [NOT_H_FILE]
    lea rdi, [rax + 2 * rdi]

    test rdi, qword [r10]
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

    pop rbp
    pop rbx
    ret

