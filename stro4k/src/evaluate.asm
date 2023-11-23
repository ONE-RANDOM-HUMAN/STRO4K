MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 29
EG_BISHOP_PAIR equ 96

MG_TEMPO equ 29
EG_TEMPO equ 9

section .rodata
MATERIAL_EVAL:
    dw  112,  212
    dw  321,  474
    dw  351,  488
    dw  469,  875
    dw 1282, 1273

; For smaller size
BISHOP_PAIR_EVAL:
    dw MG_BISHOP_PAIR, EG_BISHOP_PAIR

TEMPO_EVAL:
    db MG_TEMPO, EG_TEMPO

MOBILITY_EVAL:
    db    8,   11
    db    7,    8
    db    4,    4
    db    2,   11

PASSED_PAWN_EVAL:
    db   -3,  -14
    db  -12,    0
    db  -10,   29
    db   16,   50
    db   47,   66
    db   58,   86


; first two in each row are doubled mg and eg
; second two are isolated mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db  -49,  -85,    1,    9
    db  -25,  -57,  -15,  -10
    db  -22,  -38,  -13,  -13
    db  -29,  -27,  -26,  -22
    db  -15,  -36,  -24,  -23
    db  -29,  -48,  -22,   -7
    db  -17,  -60,  -17,   -7
    db  -30,  -81,  -24,   12


OPEN_FILE_EVAL:
    db   -3,   -9
    db   -7,    2
    db   32,    3
    db  -11,    9
    db  -50,  -10

SEMI_OPEN_FILE_EVAL:
    db   -4,   12
    db   -8,   28
    db   16,   17
    db    0,   16
    db  -16,   18

; 0-4 pawns, 4 is max possible
PAWN_SHIELD_EVAL:
    db  -34,  -20
    db  -12,  -19
    db    3,    6
    db   21,   35
    db   19,   33

; 0-6 pawns, 6 is max possible
PAWN_STORM_EVAL:
    db   10,   -5
    db    2,   18
    db  -12,   -6
    db  -25,  -44
    db    1,  -11
    db    0,    0
    db    0,    0

EVAL_WEIGHTS:
PAWN_DEFENDED_EVAL:
    db   14,   10
    db    2,   11
    db    0,   18
    db    4,   20
    db   -7,   29
    db  -45,   34

PAWN_ATTACKED_EVAL:
    db    6,   21
    db  -52,  -48
    db  -50,  -59
    db  -47,  -50
    db  -40,  -12
    db    0,    0

RANK_PST:
    db    0,    0
    db  -17,  -24
    db  -23,  -36
    db    3,  -36
    db   20,  -25
    db   38,   13
    db   58,   86
    db    0,    0


    db  -30,   -8
    db  -14,   -7
    db  -13,   -3
    db    6,   17
    db   24,   25
    db   70,    7
    db   51,    8
    db  -66,   21


    db  -14,    1
    db   -1,    0
    db    3,    1
    db    4,    5
    db    7,   11
    db   42,    5
    db   -2,   13
    db  -46,   26


    db  -17,   -6
    db  -29,    0
    db  -23,    4
    db  -17,   19
    db    7,   26
    db   36,   24
    db   44,   34
    db   62,   17


    db    1,  -13
    db   12,  -20
    db    2,    3
    db   -4,   30
    db    0,   50
    db   29,   60
    db   16,   74
    db   50,   29


    db   13,  -56
    db   -4,  -18
    db  -33,    1
    db  -36,   21
    db  -11,   42
    db   39,   58
    db   57,   44
    db   62,    7


FILE_PST:
    db  -26,    1
    db   -3,   21
    db  -16,    2
    db    4,   -9
    db    6,   -2
    db   22,   -4
    db   14,    9
    db   -9,  -20


    db  -15,    1
    db    1,    4
    db   -7,    6
    db    6,   14
    db    5,   12
    db    5,    0
    db   10,   12
    db    0,    4


    db    5,    6
    db    9,    5
    db   -4,    6
    db   -7,   10
    db   -5,    6
    db   -7,    7
    db   14,    4
    db   15,   -1


    db  -17,   21
    db  -16,   22
    db    1,   23
    db   12,   16
    db   12,    8
    db   -4,   17
    db    3,    9
    db    5,    0


    db    1,   15
    db    1,   21
    db    3,   28
    db    1,   38
    db    1,   41
    db    8,   43
    db   24,   36
    db   38,   41


    db   37,  -35
    db   34,   -2
    db   -3,   13
    db  -54,   27
    db   -5,    8
    db  -60,   21
    db   23,   -8
    db   19,  -34


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

    xor eax, eax
    mov edx, 4
.side_phase_head:
    popcnt rdi, qword [r11 + 8 * rdx]
    lea eax, [rdi + 2 * rax]
    dec edx
    jnz .side_phase_head

    add edi, eax
    push rdi

    mov rax, qword [r10] ; pawns
    mov rbx, qword [r11] ; enemy pawns
    mov rdx, qword [r10 + 40] ; king

    cmp r11, r10
    ja .pawn_shield_white

    bswap rax
    bswap rbx
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

    shl ecx, 8
    and ecx, ebx
    popcnt eax, ecx
    vpmovsxbw xmm2, qword [rbp + PAWN_STORM_EVAL - EVAL_WEIGHTS + 2 * rax]
    vpaddw xmm1, xmm2, xmm1

    vpbroadcastw xmm2, word [rsp]
    vpmullw xmm1, xmm2, xmm1
    vpsraw xmm1, xmm1, 4
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

    ; phase
    pop rcx
    pop rax
    add eax, ecx

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

