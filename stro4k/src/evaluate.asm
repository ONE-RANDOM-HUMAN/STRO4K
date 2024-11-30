MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 24
EG_BISHOP_PAIR equ 82

MG_TEMPO equ 45
EG_TEMPO equ 17

section .rodata
MATERIAL_EVAL:
    dw  117,  182
    dw  309,  500
    dw  337,  505
    dw  422,  876
    dw 1015, 1560

; For smaller size
BISHOP_PAIR_EVAL:
    dw MG_BISHOP_PAIR, EG_BISHOP_PAIR

TEMPO_EVAL:
    db MG_TEMPO, EG_TEMPO

MOBILITY_EVAL:
    db   12,    8
    db    8,    7
    db    4,    6
    db    5,    2

; stored in reverse, with rooks attacked first
MOBILITY_ATTACK_EVAL:
    db   64,   12
    db   40,   36
    db   -7,   -5
    db   -9,   28


    db   37,   41
    db   -0,    5
    db   21,   42
    db    3,   23


    db   -5,   23
    db   18,   45
    db   16,   35
    db   -1,   32


    db  -26,   43
    db   -6,   43
    db    8,   22
    db    1,   11


; first two in each row and unblocked mg and eg
; second two are blocked mg and eg
PASSED_PAWN_EVAL:
    db  -17,   -5,  -17,    0
    db  -18,   10,  -31,    9
    db  -14,   39,  -17,   15
    db    5,   65,    7,   25
    db   26,  108,   30,   28
    db   45,  119,    3,   -0


; first two in each row are doubled mg and eg
; second two are isolated mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db  -42,  -76,    3,    8
    db  -22,  -50,   -9,  -19
    db  -16,  -30,  -17,  -11
    db  -26,  -19,  -18,  -23
    db  -20,  -24,  -20,  -24
    db  -29,  -35,  -14,  -10
    db   -2,  -57,   -8,  -15
    db  -31,  -72,  -19,    5


OPEN_FILE_EVAL:
    db    3,   -7
    db   -8,    6
    db   28,   -2
    db  -12,   14
    db  -65,   -4

SEMI_OPEN_FILE_EVAL:
    db    0,    6
    db   -7,   21
    db   14,   -9
    db    7,   -1
    db  -26,   17

; 0-4 pawns, 4 is max possible
PAWN_SHIELD_EVAL:
    db   -2,  -36
    db   14,  -22
    db   17,    8
    db   23,   24
    db   23,   27

EVAL_WEIGHTS:
PAWN_DEFENDED_EVAL:
    db   15,    7
    db   -3,    9
    db    2,   15
    db    9,   10
    db   -3,   27
    db  -42,   26

PAWN_ATTACKED_EVAL:
    db   -0,   13
    db  -57,  -46
    db  -55,  -75
    db  -44,  -47
    db  -48,   -6
    db    0,    0

RANK_PST:
    db    0,    0
    db  -22,  -23
    db  -34,  -32
    db  -17,  -28
    db    0,  -21
    db   28,   -6
    db   49,  119
    db    0,    0


    db  -44,  -14
    db  -32,   -4
    db  -29,   -6
    db   -2,    9
    db   20,   10
    db   44,    1
    db   26,   -3
    db -100,   -1


    db  -22,   -6
    db  -10,   -4
    db   -3,   -6
    db   -3,   -6
    db    0,   -2
    db   22,   -0
    db  -21,   10
    db  -73,   16


    db  -24,  -17
    db  -33,   -8
    db  -32,   -3
    db  -26,   14
    db   -1,    9
    db   27,    5
    db   24,    7
    db   43,    1


    db   -5,  -58
    db   -3,  -42
    db  -14,  -10
    db  -18,   19
    db   -7,   36
    db   17,   38
    db   -9,   40
    db   32,  -21


    db    3,  -68
    db  -16,  -25
    db  -44,   -8
    db  -54,   11
    db  -35,   36
    db   -4,   56
    db   70,   33
    db   91,  -27


FILE_PST:
    db  -19,    1
    db  -10,   25
    db  -14,    3
    db   -6,   -6
    db   -1,    1
    db   21,   -4
    db    7,    7
    db    1,  -15


    db  -20,  -14
    db  -24,    1
    db  -21,   -0
    db   -5,    9
    db  -10,   10
    db  -19,    5
    db   -7,   -4
    db   -6,   -6


    db   -6,   -6
    db  -10,   -2
    db  -17,    1
    db  -17,    3
    db  -17,    5
    db  -25,    3
    db   -8,    3
    db   -5,   -7


    db  -22,    8
    db  -23,    5
    db   -6,    5
    db    3,    1
    db    7,   -6
    db  -11,    3
    db   -3,   -1
    db   -9,   -9


    db   -4,  -33
    db   -7,  -19
    db   -7,    5
    db  -11,   17
    db  -10,   19
    db   -2,   18
    db   10,    3
    db   25,   -7


    db   45,  -37
    db   26,    2
    db   -1,   13
    db  -44,   28
    db  -10,   15
    db  -55,   23
    db   23,   -7
    db   23,  -33


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

