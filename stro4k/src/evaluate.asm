MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 20
EG_BISHOP_PAIR equ 85

MG_TEMPO equ 44
EG_TEMPO equ 17

section .rodata
MATERIAL_EVAL:
    dw  127,  174
    dw  406,  488
    dw  435,  494
    dw  597,  838
    dw 1193, 1452

; For smaller size
BISHOP_PAIR_EVAL:
    dw MG_BISHOP_PAIR, EG_BISHOP_PAIR

TEMPO_EVAL:
    db MG_TEMPO, EG_TEMPO

MOBILITY_EVAL:
    db   10,   10
    db    7,    8
    db    4,    6
    db    4,    4

; stored in reverse, with rooks attacked first
MOBILITY_ATTACK_EVAL:
    db   69,   16
    db   42,   33
    db   -3,  -10
    db   -9,   25


    db   36,   40
    db   -1,   -3
    db   23,   39
    db    2,   24


    db  -11,   16
    db   16,   40
    db   17,   37
    db   -0,   31


    db  -26,   43
    db   -8,   49
    db    8,   21
    db    2,   14


; first two in each row and unblocked mg and eg
; second two are blocked mg and eg
PASSED_PAWN_EVAL:
    db  -12,    0,  -15,    8
    db  -17,   12,  -24,   11
    db  -14,   38,  -16,   16
    db    7,   65,    8,   20
    db   30,  108,   33,   27
    db   62,  127,   13,    9 ; tuner gave 62, 128, 13, 9


; first two in each row are doubled mg and eg
; second two are isolated mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db  -47,  -78,    3,   10
    db  -21,  -54,   -5,  -17
    db  -16,  -29,  -18,  -10
    db  -31,  -17,  -19,  -23
    db  -17,  -20,  -21,  -21
    db  -29,  -35,  -20,   -8
    db   -2,  -52,  -10,  -16
    db  -31,  -70,  -15,    5


OPEN_FILE_EVAL:
    db   -1,   -7
    db  -10,    8
    db   28,   -3
    db  -12,   15
    db  -65,   -5

SEMI_OPEN_FILE_EVAL:
    db   -0,    9
    db   -6,   24
    db   16,   -9
    db    8,    0
    db  -27,   15

; 0-4 pawns, 4 is max possible
PAWN_SHIELD_EVAL:
    db   21,  -31
    db   29,  -17
    db   33,   10
    db   39,   31
    db   39,   30

EVAL_WEIGHTS:
PAWN_DEFENDED_EVAL:
    db   13,    6
    db   -2,   12
    db    0,   14
    db   12,   12
    db   -4,   30
    db  -32,   35

PAWN_ATTACKED_EVAL:
    db   10,   15
    db  -56,  -45
    db  -57,  -69
    db  -46,  -44
    db  -56,  -12
    db    0,    0

RANK_PST:
    db    0,    0
    db  -16,  -22
    db  -29,  -28
    db  -12,  -26
    db    5,  -20
    db   36,   -2
    db   51,  109
    db    0,    0


    db  -80,  -14
    db  -67,  -12
    db  -61,  -16
    db  -33,    5
    db   -9,    1
    db   12,  -10
    db    3,   -4
    db -128,   12 ; tuner gave -136


    db  -55,   -8
    db  -42,   -9
    db  -35,  -13
    db  -35,  -15
    db  -28,  -10
    db  -13,   -9
    db  -49,    3
    db -100,   13


    db -109,   -8
    db -122,   -1
    db -114,    4
    db -110,   16
    db  -87,   19
    db  -60,   10
    db  -62,   12
    db  -37,   10


    db  -12,  -57
    db   -7,  -46
    db  -21,  -16
    db  -25,   16
    db  -23,   29
    db   12,   30
    db  -18,   38
    db   30,   -9


    db    4,  -54
    db  -11,  -18
    db  -29,    1
    db  -44,   19
    db  -23,   45
    db    9,   63
    db   78,   41
    db   97,  -11


FILE_PST:
    db  -33,    3
    db  -24,   28
    db  -27,    5
    db  -19,   -4
    db  -13,    1
    db    8,   -1
    db   -6,   10
    db  -14,  -14


    db  -56,  -25
    db  -54,  -13
    db  -50,  -15
    db  -37,   -4
    db  -42,   -4
    db  -47,  -14
    db  -39,   -7
    db  -45,  -15


    db  -39,  -11
    db  -35,   -7
    db  -44,  -10
    db  -44,   -5
    db  -44,   -6
    db  -51,   -6
    db  -35,   -5
    db  -32,  -11


    db  -64,    5
    db  -62,   10
    db  -47,    5
    db  -37,    2
    db  -38,   -5
    db  -51,    5
    db  -44,   -3
    db  -49,   -9


    db  -16,  -38
    db  -20,  -26
    db  -17,   -8
    db  -18,   -4
    db  -18,    6
    db  -16,   -0
    db    6,   -6
    db   21,  -18


    db   30,  -39
    db   17,    0
    db  -10,   16
    db  -37,   28
    db    4,    9
    db  -66,   23
    db   15,   -7
    db   13,  -35


default rel
section .text

; board - rsi
evaluate:
    push rbx
    push rbp
    push r13
    push r14
    push r15
    lea rbp, [EVAL_WEIGHTS]

    ; Side to move
    mov r10, rsi
    lea r11, [rsi + Board.black_pieces]
    lea r15, [rsi + Board.black]

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

    vpmovsxbw xmm0, qword [rbp + TEMPO_EVAL - EVAL_WEIGHTS]

    ; r9 - occ
    mov r9, qword [rsi + Board.white]
    or r9, qword [rsi + Board.black]

    cmp byte [rsi + Board.side_to_move], 0
    je .white_to_move

.side_eval_head:
    xchg r10, r11
    xchg r13, r14
    xor r15, 8
.white_to_move:

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

    ; exclude enemy pawn attacks
    andn rax, r14, rax

    vpmovsxbw xmm1, qword [rbp + MOBILITY_EVAL - EVAL_WEIGHTS + 2 * rcx - 2]

    popcnt rdi, rax
%ifdef AVX512
    vpbroadcastw xmm2, edi
%else
    vmovd xmm2, edi
    vpbroadcastw xmm2, xmm2
%endif
    vpmullw xmm2, xmm1, xmm2

    ; Alternatively with multiplication in GPRs
    ; EG << 16 + MG
    ; movsx edx, word [rbp + MOBILITY_EVAL - EVAL_WEIGHTS + 2 * rcx - 2]
    ; shl edx, 8
    ; xchg dl, dh
    ;
    ; imul edx, eax
    ; vmovd xmm1, edx
    vpaddw xmm0, xmm0, xmm2

    ; mobility attacks
    vpmovsxbw xmm1, qword [rbp + MOBILITY_ATTACK_EVAL - EVAL_WEIGHTS - 8 + rcx * 8]

    mov edx, 4
.mobility_attack_head:
    mov rdi, qword [r11 + rdx * 8 - 8]
    and rdi, rax
    popcnt rdi, rdi

%ifdef AVX512
    vpbroadcastw xmm2, edi
%else
    vmovd xmm2, edi
    vpbroadcastw xmm2, xmm2
%endif
    vpmullw xmm2, xmm1, xmm2
    vpaddw xmm0, xmm0, xmm2

    vpshufd xmm1, xmm1, 39h

    dec edx
    jnz .mobility_attack_head

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
    vpmovsxbw xmm2, qword [rbp + PASSED_PAWN_EVAL - EVAL_WEIGHTS + 4 * rax - 4]

    test ecx, ecx
    jnz .not_pawn_eval

    ; Doubled, isolated, and passed pawns

    ; rdi - mask in front of pawn
    ; rdx - file mask
    ; rax - stop square
    shlx rdi, r8, rbx
    shlx rdx, r8, rdx
    lea eax, [rbx + 8]
    cmp r11, r10
    ja .pawn_eval_white_piece

    xor rdi, rdx
    sub eax, 16
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

    bt qword [r15], rax
    jnc .no_blocked_passed_pawn

    vpshufd xmm2, xmm2, 01h
.no_blocked_passed_pawn:

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

    neg eax
    js .no_enemy_insufficient_material
    cmp qword [r10], 0
    jne .no_enemy_insufficient_material
    cmp eax, 700
    jg .no_enemy_insufficient_material
    sar eax, 2
.no_enemy_insufficient_material:

    neg eax
    js .no_side_insufficient_material
    cmp qword [r11], 0
    jne .no_side_insufficient_material
    cmp eax, 700
    jg .no_side_insufficient_material
    sar eax, 2
.no_side_insufficient_material:

    pop r15
    pop r14
    pop r13
    pop rbp
    pop rbx
    ret

