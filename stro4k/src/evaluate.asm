MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 28
EG_BISHOP_PAIR equ 97

MG_TEMPO equ 29
EG_TEMPO equ 9

section .rodata
MATERIAL_EVAL:
    dw  115,  214
    dw  323,  474
    dw  352,  488
    dw  470,  871
    dw 1280, 1271

; For smaller size
BISHOP_PAIR_EVAL:
    dw MG_BISHOP_PAIR, EG_BISHOP_PAIR

TEMPO_EVAL:
    db MG_TEMPO, EG_TEMPO

MOBILITY_EVAL:
    db    8,   11
    db    7,    8
    db    4,    5
    db    2,   11

; first two in each row and unblocked mg and eg
; second two are blocked mg and eg
PASSED_PAWN_EVAL:
    db   -3,  -14,   -4,   -4
    db  -11,   -1,  -19,    1
    db   -9,   30,  -17,    6
    db   17,   57,   17,    3
    db   50,   75,   50,   -2
    db   87,   93,    9,   12


; first two in each row are doubled mg and eg
; second two are isolated mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db  -49,  -83,    0,    9
    db  -25,  -56,  -15,  -10
    db  -22,  -36,  -13,  -13
    db  -29,  -25,  -26,  -21
    db  -15,  -33,  -24,  -22
    db  -28,  -46,  -22,   -7
    db  -16,  -58,  -17,   -7
    db  -28,  -80,  -25,   12


OPEN_FILE_EVAL:
    db   -3,   -8
    db   -7,    3
    db   32,   -1
    db  -11,    8
    db  -51,   -9

SEMI_OPEN_FILE_EVAL:
    db   -3,    5
    db   -7,   21
    db   16,   11
    db    0,   12
    db  -15,   12

; 0-4 pawns, 4 is max possible
PAWN_SHIELD_EVAL:
    db  -29,  -24
    db   -8,  -21
    db    7,    5
    db   25,   33
    db   21,   34

EVAL_WEIGHTS:
PAWN_DEFENDED_EVAL:
    db   14,   11
    db    2,   13
    db    0,   19
    db    5,   18
    db   -6,   29
    db  -45,   35

PAWN_ATTACKED_EVAL:
    db    6,   21
    db  -52,  -48
    db  -50,  -60
    db  -44,  -47
    db  -40,  -11
    db    0,    0

RANK_PST:
    db    0,    0
    db  -21,  -27
    db  -27,  -40
    db   -1,  -40
    db   18,  -30
    db   36,   12
    db   82,   92
    db    0,    0


    db  -30,  -10
    db  -14,   -8
    db  -13,   -3
    db    6,   18
    db   24,   27
    db   70,    8
    db   51,    8
    db  -66,   21


    db  -14,    0
    db   -1,   -1
    db    3,    0
    db    4,    5
    db    6,   12
    db   42,    6
    db   -2,   15
    db  -45,   28


    db  -17,   -7
    db  -29,   -2
    db  -23,    4
    db  -18,   19
    db    6,   27
    db   35,   25
    db   44,   34
    db   62,   16


    db    1,  -13
    db   13,  -22
    db    2,    2
    db   -4,   30
    db   -1,   50
    db   29,   61
    db   15,   73
    db   50,   30


    db   14,  -55
    db   -2,  -19
    db  -35,    0
    db  -39,   21
    db  -13,   43
    db   37,   58
    db   53,   42
    db   55,    4


FILE_PST:
    db  -26,    2
    db   -3,   21
    db  -16,    2
    db    4,   -8
    db    6,   -2
    db   22,   -4
    db   13,    9
    db   -9,  -20


    db  -15,   -1
    db    1,    4
    db   -6,    6
    db    6,   14
    db    5,   13
    db    5,    1
    db    9,   13
    db    0,    3


    db    5,    4
    db    9,    4
    db   -4,    6
    db   -7,   10
    db   -5,    6
    db   -8,    8
    db   14,    5
    db   15,   -1


    db  -17,   21
    db  -16,   22
    db    1,   23
    db   13,   16
    db   13,    8
    db   -3,   16
    db    3,    9
    db    4,    1


    db    1,   15
    db    1,   21
    db    3,   27
    db    1,   37
    db    1,   41
    db    8,   43
    db   24,   36
    db   38,   42


    db   38,  -37
    db   35,   -3
    db   -1,   13
    db  -56,   26
    db  -10,    9
    db  -59,   22
    db   25,   -7
    db   20,  -34


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

