MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 27
EG_BISHOP_PAIR equ 95

MG_TEMPO equ 30
EG_TEMPO equ 14

section .rodata
MATERIAL_EVAL:
    dw  114,  178
    dw  366,  488
    dw  412,  487
    dw  553,  854
    dw 1381, 1427

; For smaller size
BISHOP_PAIR_EVAL:
    dw MG_BISHOP_PAIR, EG_BISHOP_PAIR

TEMPO_EVAL:
    db MG_TEMPO, EG_TEMPO

MOBILITY_EVAL:
    db   13,    8
    db    8,    8
    db    5,    6
    db    4,    4

; stored in reverse, with rooks attacked first
MOBILITY_ATTACK_EVAL:
    db   76,   11
    db   50,   33
    db   -1,   -8
    db   -6,   27


    db   47,   40
    db    3,    0
    db   18,   53
    db    3,   29


    db   -5,   19
    db   18,   41
    db   21,   34
    db    2,   34


    db  -22,   26
    db   -5,   42
    db    8,   23
    db    2,   14


; first two in each row and unblocked mg and eg
; second two are blocked mg and eg
PASSED_PAWN_EVAL:
    db  -11,   -4,  -13,    4
    db  -17,    9,  -26,   10
    db  -10,   41,  -20,   19
    db   21,   67,   17,   14
    db   46,  107,   48,   25
    db  102,  124,   16,   29


; first two in each row are doubled mg and eg
; second two are isolated mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db  -56,  -87,    4,    9
    db  -33,  -51,  -14,  -14
    db  -26,  -28,  -14,  -17
    db  -37,  -16,  -30,  -24
    db  -20,  -28,  -22,  -28
    db  -34,  -37,  -24,   -9
    db  -12,  -58,  -14,  -15
    db  -33,  -80,  -26,   11


OPEN_FILE_EVAL:
    db    1,  -10
    db   -6,    3
    db   34,   -4
    db  -11,   18
    db  -56,   -6

SEMI_OPEN_FILE_EVAL:
    db    0,    7
    db   -6,   22
    db   21,  -10
    db    3,    5
    db  -15,   10

; 0-4 pawns, 4 is max possible
PAWN_SHIELD_EVAL:
    db  -32,  -23
    db   -9,  -21
    db    6,    6
    db   23,   35
    db   16,   49

EVAL_WEIGHTS:
PAWN_DEFENDED_EVAL:
    db   15,    7
    db    1,   10
    db    2,   15
    db   11,   16
    db   -2,   29
    db  -49,   38

PAWN_ATTACKED_EVAL:
    db    8,   31
    db  -67,  -50
    db  -50,  -78
    db  -53,  -36
    db  -54,   -9
    ; db    0,    0

RANK_PST:
    db    0,    0
    db  -11,   -7
    db  -22,  -18
    db    1,  -17
    db   18,   -6
    db   48,   19
    db   83,  119
    db    0,    0


    db  -27,    0
    db  -13,    4
    db  -14,   10
    db   18,   31
    db   38,   30
    db   74,   14
    db   63,   13
    db  -81,   31


    db  -10,   13
    db    7,   10
    db   11,   13
    db   12,   15
    db   11,   22
    db   42,   17
    db   -1,   22
    db  -44,   35


    db  -10,   15
    db  -21,   21
    db  -18,   26
    db  -10,   40
    db   17,   42
    db   49,   33
    db   41,   37
    db   71,   30


    db   19,    6
    db   27,   10
    db   12,   48
    db    7,   80
    db   12,   98
    db   44,   93
    db   17,  105
    db   59,   73


    db   15,  -60
    db   -2,  -19
    db  -39,    0
    db  -43,   25
    db  -20,   47
    db   33,   65
    db   69,   52
    db  104,    1


FILE_PST:
    db  -21,    7
    db   -2,   26
    db  -17,    8
    db    2,   -3
    db    4,    7
    db   27,   -1
    db   17,   15
    db    0,  -15


    db   -9,    0
    db    3,    9
    db   -4,   19
    db   13,   27
    db    7,   27
    db    2,   15
    db   12,   19
    db    9,    5


    db   12,    8
    db   15,   12
    db    0,   19
    db    1,   22
    db    0,   22
    db   -5,   23
    db   20,   13
    db   20,    6


    db  -17,   36
    db  -12,   37
    db    4,   38
    db   17,   30
    db   18,   22
    db    0,   32
    db    7,   25
    db    6,   13


    db   16,   29
    db   16,   46
    db   13,   63
    db    9,   77
    db   11,   77
    db   19,   76
    db   40,   56
    db   54,   50


    db   43,  -39
    db   38,   -2
    db   -1,   16
    db  -67,   30
    db  -14,   10
    db  -62,   22
    db   24,  -11
    db   22,  -39


default rel
section .text

; board - rsi
evaluate:
    push r15
    push r14
    push r13
    push rbp
    push rbx
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
    ; andn r14, r8, rdi
    ; or r14, rax
    ; mov r13, r14

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
    ; neg rbx
    ; test rax, rbx
    andn rax, rbx, rax
    jz .no_bishop_pair

    vpaddw xmm0, xmm0, oword [rbp + BISHOP_PAIR_EVAL - EVAL_WEIGHTS]
.no_bishop_pair:

    ; Pawn shield

    xor eax, eax
    ; mov edx, 4
    push 4
    pop rdx
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

    shl ecx, 5 ; 0E0E0h
    test edx, ecx
    jz .no_pawn_shield
.pawn_shield:
    shl ecx, 8
    and eax, ecx
    popcnt eax, eax

    vpmovsxbw xmm1, qword [rbp + PAWN_SHIELD_EVAL - EVAL_WEIGHTS + 2 * rax]
    vpbroadcastw xmm2, word [rsp]
    vpmullw xmm1, xmm1, xmm2
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
    mov rdi, rax
    and rdi, qword [r11 + rdx * 8 - 8]
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

    pop rbx
    pop rbp
    pop r13
    pop r14
    pop r15
    ret

