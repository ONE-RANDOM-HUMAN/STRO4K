MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 46
EG_BISHOP_PAIR equ 90

MG_TEMPO equ 38
EG_TEMPO equ 4

section .rodata
MATERIAL_EVAL:
    dw  166,  154
    dw  446,  355
    dw  483,  346
    dw  692,  657
    dw 1570, 1122

; For smaller size
BISHOP_PAIR_EVAL:
    dw MG_BISHOP_PAIR, EG_BISHOP_PAIR

TEMPO_EVAL:
    db MG_TEMPO, EG_TEMPO

MOBILITY_EVAL:
    db   13,    5
    db   11,    5
    db    7,    1
    db    6,   -1

PASSED_PAWN_EVAL:
    db  -11,  -10
    db  -18,    4
    db  -10,   25
    db   27,   34
    db   64,   42
    db   74,   67


; first two in each row are doubled mg and eg
; second two are isolated mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db  -70,  -62,    1,    3
    db  -40,  -42,  -16,   -9
    db  -36,  -26,  -18,  -13
    db  -43,  -17,  -40,  -14
    db  -33,  -22,  -32,  -24
    db  -49,  -30,  -23,  -11
    db  -31,  -40,  -13,  -14
    db  -54,  -50,  -24,   -2


OPEN_FILE_EVAL:
    db   -3,  -13
    db  -10,   -2
    db   43,   -5
    db  -14,   17
    db  -57,   -6

SEMI_OPEN_FILE_EVAL:
    db   -2,    6
    db   -8,   21
    db   24,    3
    db    5,    6
    db  -18,   15

; 0-4 pawns, 4 is max possible
PAWN_SHIELD_EVAL:
    db  -59,   22
    db  -19,   -4
    db   19,  -10
    db   55,   -9
    db   37,    9

EVAL_WEIGHTS:
PAWN_DEFENDED_EVAL:
    db   19,    8
    db    1,   15
    db    1,   19
    db    2,   29
    db   -8,   34
    db  -34,   33

PAWN_ATTACKED_EVAL:
    db   10,   25
    db  -66,  -39
    db  -62,  -51
    db  -54,  -42
    db  -47,  -24
    db  -43,  -12

RANK_PST:
    db    0,    0
    db  -21,    1
    db  -29,  -12
    db    8,  -19
    db   30,   -9
    db   55,   16
    db   74,   67
    db    0,    0


    db  -40,  -26
    db  -16,  -22
    db  -19,   -8
    db    6,   15
    db   32,   21
    db   72,    6
    db   61,    0
    db  -48,    5


    db  -17,  -12
    db   -2,   -9
    db    5,   -5
    db    1,    3
    db    4,   11
    db   46,   -1
    db   -3,    5
    db  -40,   14


    db  -20,  -21
    db  -38,  -14
    db  -31,   -7
    db  -20,    5
    db   15,    9
    db   46,    7
    db   57,   16
    db   63,    5


    db   -6,  -59
    db    3,  -57
    db  -13,  -11
    db  -19,   26
    db   -6,   43
    db   30,   37
    db   11,   46
    db   55,    4


    db    1,  -37
    db  -13,   -9
    db  -42,    5
    db   -9,   13
    db   27,   20
    db   55,   30
    db   66,   26
    db   61,    3


FILE_PST:
    db  -24,    6
    db   -3,   17
    db  -20,    9
    db    6,   -8
    db    6,    2
    db   26,   -7
    db   21,   -4
    db   -9,  -19


    db  -20,  -16
    db   -3,   -5
    db   -7,    3
    db    5,    9
    db    4,    8
    db    5,   -5
    db    9,    2
    db   -1,  -10


    db    1,   -3
    db    8,   -2
    db   -7,    1
    db   -8,    5
    db   -9,    3
    db  -13,    2
    db   19,   -6
    db   19,  -10


    db  -20,    5
    db  -16,    5
    db    4,    5
    db   16,    1
    db   17,   -9
    db   -1,   -1
    db    4,   -3
    db   -4,  -10


    db   -8,  -38
    db   -8,  -14
    db   -5,    1
    db   -8,   17
    db   -7,   19
    db   -1,   15
    db   26,   -6
    db   37,  -17


    db   25,  -18
    db   38,   -2
    db    1,    8
    db  -50,   18
    db   -7,    4
    db  -60,   14
    db   26,   -7
    db   15,  -22


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

