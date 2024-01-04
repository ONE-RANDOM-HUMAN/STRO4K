MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 28
EG_BISHOP_PAIR equ 92

MG_TEMPO equ 31
EG_TEMPO equ 15

section .rodata
MATERIAL_EVAL:
    dw  110,  220
    dw  328,  500
    dw  372,  504
    dw  489,  867
    dw 1309, 1303

; For smaller size
BISHOP_PAIR_EVAL:
    dw MG_BISHOP_PAIR, EG_BISHOP_PAIR

TEMPO_EVAL:
    db MG_TEMPO, EG_TEMPO

MOBILITY_EVAL:
    db   12,    8
    db    8,    8
    db    4,    7
    db    4,   10

; stored in reverse, with rooks attacked first
MOBILITY_ATTACK_EVAL:
    db   66,   18
    db   47,   30
    db    0,   -6
    db   -4,   25


    db   44,   43
    db    4,    2
    db   19,   52
    db    3,   30


    db   -3,   18
    db   19,   40
    db   21,   33
    db    3,   32


    db  -25,   36
    db   -4,   47
    db    7,   27
    db    0,   20


; first two in each row and unblocked mg and eg
; second two are blocked mg and eg
PASSED_PAWN_EVAL:
    db   -8,  -14,  -10,   -4
    db  -15,    1,  -19,   -3
    db  -10,   32,  -14,    7
    db   21,   56,   17,    4
    db   52,   76,   48,   -1
    db   90,   96,   10,   12


; first two in each row are doubled mg and eg
; second two are isolated mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db  -53,  -85,    2,   10
    db  -28,  -57,  -15,  -10
    db  -21,  -38,  -13,  -13
    db  -32,  -26,  -28,  -20
    db  -16,  -33,  -23,  -22
    db  -28,  -47,  -23,   -8
    db  -13,  -60,  -16,   -9
    db  -30,  -82,  -24,   10


OPEN_FILE_EVAL:
    db    0,   -8
    db   -6,    4
    db   33,   -3
    db  -12,    9
    db  -55,   -8

SEMI_OPEN_FILE_EVAL:
    db   -1,    9
    db   -6,   24
    db   20,   -9
    db    2,    8
    db  -16,   11

; 0-4 pawns, 4 is max possible
PAWN_SHIELD_EVAL:
    db  -27,  -23
    db   -8,  -22
    db    9,    4
    db   25,   30
    db   19,   35

EVAL_WEIGHTS:
PAWN_DEFENDED_EVAL:
    db   14,    6
    db    2,    9
    db    2,   13
    db   10,   17
    db   -2,   36
    db  -43,   35

PAWN_ATTACKED_EVAL:
    db    7,   23
    db  -65,  -47
    db  -53,  -75
    db  -50,  -50
    db  -49,  -19
    db    0,    0

RANK_PST:
    db    0,    0
    db  -15,  -30
    db  -26,  -41
    db   -4,  -41
    db   13,  -29
    db   36,   11
    db   85,   94
    db    0,    0


    db  -32,  -14
    db  -17,   -6
    db  -18,   -1
    db   14,   18
    db   33,   19
    db   61,    7
    db   51,    6
    db  -60,    7


    db  -15,    1
    db    1,   -1
    db    5,    1
    db    6,    3
    db    7,   11
    db   32,    6
    db   -7,   12
    db  -48,   20


    db  -13,   -5
    db  -24,    2
    db  -20,    8
    db  -13,   21
    db   10,   22
    db   37,   17
    db   33,   22
    db   53,   15


    db    1,  -11
    db   10,  -16
    db   -2,   11
    db   -3,   38
    db    3,   55
    db   33,   57
    db   13,   61
    db   42,   27


    db   13,  -56
    db   -1,  -18
    db  -36,   -1
    db  -37,   21
    db  -12,   44
    db   38,   59
    db   58,   42
    db   57,    4


FILE_PST:
    db  -20,    3
    db   -4,   21
    db  -16,    2
    db    0,   -9
    db    3,   -2
    db   22,   -6
    db   15,    8
    db   -5,  -20


    db  -12,  -11
    db   -2,   -1
    db   -8,    6
    db    9,   15
    db    4,   14
    db   -1,    4
    db    9,    7
    db    6,   -6


    db    4,   -1
    db    9,    2
    db   -5,    6
    db   -4,    9
    db   -6,    8
    db  -11,   11
    db   15,    2
    db   15,   -6


    db  -19,   17
    db  -15,   18
    db    2,   19
    db   14,   13
    db   15,    5
    db   -4,   14
    db    3,    9
    db    2,   -2


    db   -1,    8
    db    2,   13
    db    2,   25
    db   -2,   37
    db    0,   41
    db    7,   42
    db   25,   33
    db   38,   37


    db   37,  -38
    db   36,   -4
    db    1,   12
    db  -56,   26
    db  -11,   10
    db  -57,   22
    db   24,   -7
    db   18,  -34


default rel
section .text

; board - rsi
; returns eval in eax and 2 * phase in edx
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
    pop rdx
    add edx, ecx

    vmovd ecx, xmm0
    movsx ebx, cx
    sar ecx, 16

    imul ebx, edx

    lea eax, [rdx - 48] ; 2 * -(24 - phase)
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

    ; eax - eval, edx - 2 * phase
    ret

