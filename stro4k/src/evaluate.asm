MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 28
EG_BISHOP_PAIR equ 89

MG_TEMPO equ 31
EG_TEMPO equ 12

section .rodata
MATERIAL_EVAL:
    dw  115,  181
    dw  349,  469
    dw  396,  468
    dw  524,  824
    dw 1306, 1398

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
    db   68,   16
    db   48,   26
    db   -1,   -3
    db   -5,   23


    db   45,   40
    db    2,   -1
    db   20,   48
    db    3,   27


    db   -2,   22
    db   19,   36
    db   22,   29
    db    3,   29


    db  -23,   31
    db   -4,   39
    db    7,   19
    db    2,   12


; first two in each row and unblocked mg and eg
; second two are blocked mg and eg
PASSED_PAWN_EVAL:
    db   -8,   -6,  -11,    3
    db  -16,    6,  -21,    4
    db   -9,   34,  -19,   10
    db   24,   55,   17,    8
    db   53,   81,   48,   16
    db  105,  104,    7,   22


; first two in each row are doubled mg and eg
; second two are isolated mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db  -57,  -77,    4,    6
    db  -29,  -47,  -14,  -12
    db  -26,  -30,  -13,  -15
    db  -34,  -18,  -28,  -19
    db  -19,  -28,  -23,  -24
    db  -32,  -37,  -21,  -10
    db  -20,  -50,  -13,  -12
    db  -37,  -69,  -20,    5


OPEN_FILE_EVAL:
    db    0,   -9
    db   -6,    2
    db   34,   -5
    db  -10,   12
    db  -57,   -6

SEMI_OPEN_FILE_EVAL:
    db    1,    6
    db   -4,   20
    db   21,  -13
    db    4,    3
    db  -15,    9

; 0-4 pawns, 4 is max possible
PAWN_SHIELD_EVAL:
    db  -26,  -20
    db   -5,  -22
    db   10,    4
    db   27,   30
    db   20,   35

EVAL_WEIGHTS:
PAWN_DEFENDED_EVAL:
    db   15,    7
    db    1,    9
    db    1,   14
    db   10,   18
    db   -2,   26
    db  -35,   33

PAWN_ATTACKED_EVAL:
    db   10,   26
    db  -66,  -49
    db  -54,  -73
    db  -53,  -47
    db  -50,  -23
    db    0,    0

RANK_PST:
    db    0,    0
    db  -12,   -9
    db  -22,  -20
    db    0,  -20
    db   16,   -8
    db   45,   19
    db   91,  100
    db    0,    0


    db  -30,  -21
    db  -17,  -13
    db  -19,   -5
    db   14,   14
    db   34,   14
    db   63,    1
    db   52,   -1
    db  -66,    6


    db  -16,   -5
    db    1,   -6
    db    4,   -3
    db    4,    0
    db    5,    6
    db   32,    1
    db   -7,    6
    db  -47,   14


    db  -14,  -16
    db  -24,   -8
    db  -21,   -4
    db  -13,    9
    db   12,   10
    db   40,    5
    db   34,   10
    db   57,    2


    db   -3,  -53
    db    4,  -49
    db   -9,   -9
    db  -13,   21
    db   -6,   38
    db   24,   34
    db    2,   39
    db   36,   -4


    db   11,  -52
    db   -4,  -14
    db  -35,    1
    db  -32,   21
    db   -3,   38
    db   45,   52
    db   62,   40
    db   64,    6


FILE_PST:
    db  -21,    3
    db   -4,   19
    db  -18,    5
    db    0,   -8
    db    2,    1
    db   24,   -5
    db   20,    3
    db   -2,  -19


    db  -13,  -17
    db   -3,   -8
    db   -9,    3
    db    9,    9
    db    3,    9
    db   -1,   -2
    db    9,    1
    db    5,  -12


    db    5,   -6
    db    9,   -5
    db   -5,    1
    db   -5,    5
    db   -6,    5
    db  -12,    6
    db   14,   -4
    db   14,  -11


    db  -20,    6
    db  -15,    6
    db    1,    7
    db   13,    0
    db   16,   -6
    db   -2,    1
    db    5,   -5
    db    2,  -14


    db   -4,  -33
    db   -4,  -17
    db   -5,    2
    db  -10,   16
    db   -8,   19
    db   -2,   14
    db   19,   -6
    db   33,  -10


    db   33,  -32
    db   32,   -1
    db    0,   14
    db  -55,   25
    db   -9,    8
    db  -59,   20
    db   24,   -8
    db   19,  -34


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

