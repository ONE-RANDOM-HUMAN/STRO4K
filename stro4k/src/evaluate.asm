MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 27
EG_BISHOP_PAIR equ 90

MG_TEMPO equ 50
EG_TEMPO equ 19

section .rodata
MATERIAL_EVAL:
    dw  131,  202
    dw  354,  529
    dw  392,  535
    dw  507,  930
    dw 1208, 1647

; For smaller size
BISHOP_PAIR_EVAL:
    dw MG_BISHOP_PAIR, EG_BISHOP_PAIR

TEMPO_EVAL:
    db MG_TEMPO, EG_TEMPO

MOBILITY_EVAL:
    db   12,    8
    db    9,    7
    db    4,    7
    db    6,    1

; stored in reverse, with rooks attacked first
MOBILITY_ATTACK_EVAL:
    db   72,   16
    db   44,   34
    db   -7,    1
    db  -10,   32


    db   32,   43
    db   -7,   -2
    db   26,   48
    db    5,   31


    db    6,   36
    db   18,   50
    db   13,   36
    db   -0,   35


    db  -27,   44
    db  -10,   51
    db    4,   21
    db   -1,   10


; first two in each row and unblocked mg and eg
; second two are blocked mg and eg
PASSED_PAWN_EVAL:
    db  -17,   -0,  -13,    4
    db  -24,   15,  -37,    4
    db  -19,   42,  -24,   18
    db   11,   75,    8,   17
    db   28,  127,   31,   36 ; tuner gave 128
    db   46,  127,    5,   -2 ; tuner gave 136


; first two in each row are doubled mg and eg
; second two are isolated mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db  -55,  -89,    1,   14
    db  -21,  -64,   -8,  -20
    db  -17,  -39,  -19,  -11
    db  -27,  -17,  -24,  -22
    db  -25,  -26,  -25,  -24
    db  -34,  -40,  -26,  -10
    db    5,  -67,  -13,  -13
    db  -26,  -85,  -25,   10


OPEN_FILE_EVAL:
    db    0,   -8
    db  -10,    8
    db   33,   -3
    db  -12,   17
    db  -71,   -2

SEMI_OPEN_FILE_EVAL:
    db    1,   10
    db   -9,   25
    db   20,   -9
    db    9,   -3
    db  -32,   17

; 0-4 pawns, 4 is max possible
PAWN_SHIELD_EVAL:
    db   -2,  -39
    db   11,  -32
    db   16,   -3
    db   22,   17
    db   21,   22

EVAL_WEIGHTS:
PAWN_DEFENDED_EVAL:
    db   16,    9
    db   -3,   12
    db    2,   13
    db   14,   14
    db   -3,   35
    db  -54,   34

PAWN_ATTACKED_EVAL:
    db    6,   17
    db  -64,  -53
    db  -65,  -83
    db  -62,  -55
    db  -60,  -16
    db    0,    0

RANK_PST:
    db    0,    0
    db  -21,  -27
    db  -37,  -32
    db  -20,  -33
    db    1,  -22
    db   35,   -3
    db   53,  127 ; tuner gave 134
    db    0,    0


    db  -50,   -9
    db  -36,   -7
    db  -29,   -6
    db    2,   16
    db   27,   15
    db   47,   -3
    db   38,    5
    db -107,    8


    db  -19,   -3
    db   -8,   -4
    db   -2,   -2
    db   -1,   -5
    db    1,    1
    db   27,   -0
    db  -22,    7
    db  -83,   26


    db  -26,  -11
    db  -41,   -6
    db  -34,    1
    db  -33,   16
    db    3,   14
    db   31,    5
    db   30,   12
    db   53,    4


    db   -2,  -57
    db   -3,  -51
    db  -14,   -7
    db  -16,   26
    db  -14,   42
    db   18,   33
    db  -11,   47
    db   43,  -14


    db   13,  -71
    db   -3,  -29
    db  -50,   -8
    db  -53,   11
    db  -46,   37
    db   -9,   50
    db   60,   29
    db   87,  -21


FILE_PST:
    db  -21,    1
    db   -9,   30
    db  -13,    3
    db   -5,   -4
    db    0,   -4
    db   23,   -2
    db    2,   15
    db    2,  -18


    db  -25,  -17
    db  -19,    1
    db  -21,    7
    db   -4,   12
    db   -9,   12
    db  -16,    6
    db   -6,    2
    db  -14,   -6


    db  -12,   -6
    db   -7,    4
    db  -15,    2
    db  -14,    8
    db  -18,    7
    db  -25,    2
    db   -6,    2
    db   -4,   -1


    db  -25,    9
    db  -24,   11
    db   -5,   13
    db    3,    3
    db    2,   -5
    db  -10,    7
    db   -7,    2
    db   -8,   -2


    db   -1,  -33
    db  -10,  -11
    db  -13,   12
    db   -9,   26
    db  -11,   24
    db   -5,   19
    db   19,   -6
    db   34,  -14


    db   53,  -43
    db   30,   -3
    db   -1,   20
    db  -64,   29
    db  -11,   12
    db  -65,   25
    db   29,   -3
    db   26,  -38


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

