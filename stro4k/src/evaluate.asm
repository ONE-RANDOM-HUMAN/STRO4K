MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 27
EG_BISHOP_PAIR equ 95

MG_TEMPO equ 30
EG_TEMPO equ 14

section .rodata
MATERIAL_EVAL:
    dw  113,  195
    dw  371,  530
    dw  419,  529
    dw  548,  926
    dw 1381, 1599

; For smaller size
BISHOP_PAIR_EVAL:
    dw MG_BISHOP_PAIR, EG_BISHOP_PAIR

TEMPO_EVAL:
    db MG_TEMPO, EG_TEMPO

MOBILITY_EVAL:
    db   12,    7
    db    8,    7
    db    5,    6
    db    5,    2

; stored in reverse, with rooks attacked first
MOBILITY_ATTACK_EVAL:
    db   67,   16
    db   48,   30
    db   -3,   -5
    db   -5,   25


    db   43,   36
    db    3,   -1
    db   19,   51
    db    3,   28


    db   -3,   19
    db   20,   38
    db   21,   32
    db    3,   30


    db  -20,   21
    db   -5,   38
    db    7,   20
    db    1,   13


; first two in each row and unblocked mg and eg
; second two are blocked mg and eg
PASSED_PAWN_EVAL:
    db   -7,   -6,  -10,    0
    db  -13,    5,  -22,    4
    db   -8,   36,  -17,   11
    db   23,   61,   17,   10
    db   52,   90,   49,   16
    db  100,  109,   14,   27


; first two in each row are doubled mg and eg
; second two are isolated mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db  -56,  -79,    3,    7
    db  -28,  -50,  -14,  -12
    db  -24,  -30,  -13,  -16
    db  -34,  -20,  -30,  -21
    db  -18,  -28,  -24,  -27
    db  -30,  -40,  -23,   -9
    db  -15,  -54,  -16,  -11
    db  -34,  -75,  -23,   10


OPEN_FILE_EVAL:
    db    0,  -11
    db   -5,    3
    db   32,   -3
    db  -11,   16
    db  -52,   -7

SEMI_OPEN_FILE_EVAL:
    db   -1,    6
    db   -5,   20
    db   19,   -9
    db    3,    2
    db  -14,   10

; 0-4 pawns, 4 is max possible
PAWN_SHIELD_EVAL:
    db  -25,  -20
    db   -6,  -18
    db   10,    7
    db   26,   35
    db   22,   35

EVAL_WEIGHTS:
PAWN_DEFENDED_EVAL:
    db   14,    8
    db    0,   10
    db    2,   16
    db    8,   16
    db   -1,   25
    db  -40,   34

PAWN_ATTACKED_EVAL:
    db    7,   24
    db  -64,  -48
    db  -51,  -70
    db  -48,  -36
    db  -48,  -14
    db    0,    0

RANK_PST:
    db    0,    0
    db  -11,  -14
    db  -21,  -24
    db    1,  -23
    db   17,  -11
    db   44,   22
    db   87,  106
    db    0,    0


    db  -30,  -19
    db  -17,  -13
    db  -19,   -5
    db   12,   14
    db   33,   15
    db   64,    2
    db   52,   -1
    db  -61,    3


    db  -16,   -4
    db    0,   -7
    db    4,   -3
    db    4,    0
    db    3,    7
    db   33,    2
    db   -7,    5
    db  -40,   15


    db  -13,  -15
    db  -24,   -8
    db  -20,   -4
    db  -13,    9
    db   13,   11
    db   39,    5
    db   31,   11
    db   57,    5


    db   -2,  -53
    db    5,  -48
    db   -9,  -11
    db  -12,   20
    db   -7,   35
    db   22,   32
    db    1,   39
    db   34,   18


    db   11,  -57
    db   -4,  -17
    db  -34,    1
    db  -29,   22
    db    0,   45
    db   42,   62
    db   60,   52
    db   65,   12


FILE_PST:
    db  -20,    2
    db   -5,   19
    db  -17,    3
    db    0,   -9
    db    2,   -1
    db   24,   -4
    db   18,    5
    db   -2,  -18


    db  -13,  -18
    db   -2,   -8
    db   -9,    3
    db    9,   10
    db    3,   11
    db   -1,   -2
    db    7,    2
    db    5,  -14


    db    4,   -8
    db    8,   -4
    db   -5,    1
    db   -5,    6
    db   -7,    4
    db  -12,    7
    db   14,   -3
    db   12,  -12


    db  -20,    6
    db  -15,    6
    db    2,    8
    db   13,    0
    db   15,   -7
    db   -4,    1
    db    3,   -3
    db    2,  -15


    db   -6,  -31
    db   -4,  -13
    db   -5,    2
    db   -9,   15
    db   -7,   17
    db    0,   13
    db   18,   -4
    db   30,   -9


    db   34,  -34
    db   33,   -2
    db   -1,   14
    db  -54,   27
    db   -9,    9
    db  -58,   21
    db   23,  -10
    db   18,  -36


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
    vmovd xmm2, edi
    vpbroadcastw xmm2, xmm2
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

    vmovd xmm2, edi
    vpbroadcastw xmm2, xmm2
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

    pop r15
    pop r14
    pop r13
    pop rbp
    pop rbx
    ret

