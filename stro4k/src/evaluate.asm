MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 28
EG_BISHOP_PAIR equ 91

MG_TEMPO equ 29
EG_TEMPO equ 13

section .rodata
MATERIAL_EVAL:
    dw  109,  228
    dw  321,  504
    dw  362,  508
    dw  472,  879
    dw 1313, 1324

; For smaller size
BISHOP_PAIR_EVAL:
    dw MG_BISHOP_PAIR, EG_BISHOP_PAIR

TEMPO_EVAL:
    db MG_TEMPO, EG_TEMPO

MOBILITY_EVAL:
    db   12,    7
    db    8,    7
    db    5,    6
    db    4,    9

; stored in reverse, with rooks attacked first
MOBILITY_ATTACK_EVAL:
    db   45,   25
    db   41,   29
    db    0,   -4
    db   -3,   23


    db   37,   35
    db    3,    2
    db   19,   41
    db    3,   27


    db   -1,   16
    db   22,   32
    db   22,   27
    db    6,   26


    db  -20,   25
    db   -1,   32
    db    5,   22
    db    0,   18


; first two in each row and unblocked mg and eg
; second two are blocked mg and eg
PASSED_PAWN_EVAL:
    db   -7,  -24,  -12,  -13
    db  -13,  -12,  -16,  -14
    db   -8,   18,  -11,   -5
    db   23,   37,   16,   -7
    db   41,   41,   34,  -15
    db   50,   50,   30,   32


; first two in each row are doubled mg and eg
; second two are isolated mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db  -45,  -49,    1,    8
    db  -29,  -45,  -14,   -4
    db  -21,  -37,  -13,  -11
    db  -28,  -27,  -26,  -17
    db  -15,  -32,  -24,  -18
    db  -26,  -43,  -22,   -7
    db  -19,  -45,  -17,   -5
    db  -36,  -49,  -20,    6


OPEN_FILE_EVAL:
    db   -1,   -8
    db   -6,    3
    db   29,    0
    db  -11,    9
    db  -43,  -10

SEMI_OPEN_FILE_EVAL:
    db   -1,    9
    db   -5,   21
    db   17,  -10
    db    2,    4
    db  -15,    9

; 0-4 pawns, 4 is max possible
PAWN_SHIELD_EVAL:
    db  -23,  -24
    db   -8,  -19
    db    9,    2
    db   25,   25
    db   18,   20

EVAL_WEIGHTS:
PAWN_DEFENDED_EVAL:
    db   14,    4
    db    2,    9
    db    2,   12
    db    9,   17
    db   -1,   30
    db  -29,   31

PAWN_ATTACKED_EVAL:
    db    7,   17
    db  -49,  -41
    db  -46,  -48
    db  -41,  -43
    db  -38,  -22
    db    0,    0

RANK_PST:
    db    0,    0
    db  -17,  -29
    db  -28,  -40
    db   -7,  -41
    db    9,  -27
    db   31,   22
    db   50,   50
    db    0,    0


    db  -30,  -22
    db  -18,  -13
    db  -19,   -6
    db   11,   14
    db   28,   17
    db   43,   10
    db   37,    6
    db  -37,   -2


    db  -15,   -5
    db    0,   -5
    db    4,   -2
    db    4,    1
    db    5,    9
    db   26,    6
    db   -5,    7
    db  -31,   11


    db  -11,  -18
    db  -23,   -9
    db  -17,   -4
    db  -11,    8
    db   10,   11
    db   30,    9
    db   26,   17
    db   37,   10


    db   -5,  -30
    db    2,  -32
    db   -9,   -7
    db   -9,   15
    db    0,   28
    db   25,   30
    db   10,   31
    db   24,   11


    db    1,  -42
    db   -8,  -14
    db  -30,   -1
    db  -21,   18
    db   15,   35
    db   42,   43
    db   42,   39
    db   36,   17


FILE_PST:
    db  -18,    5
    db   -4,   18
    db  -15,    2
    db    0,   -7
    db    3,   -4
    db   20,   -6
    db   16,    3
    db   -7,  -17


    db  -12,  -15
    db   -3,   -5
    db   -7,    3
    db    9,   11
    db    4,    9
    db   -2,    1
    db    7,    3
    db    5,  -10


    db    4,   -6
    db    9,   -2
    db   -5,    3
    db   -4,    6
    db   -6,    5
    db  -11,    7
    db   13,   -1
    db   14,  -10


    db  -18,    7
    db  -13,    8
    db    4,    9
    db   15,    4
    db   15,   -3
    db   -3,    5
    db    0,    1
    db   -2,   -9


    db   -7,  -11
    db   -5,   -4
    db   -4,    5
    db   -7,   15
    db   -6,   18
    db    1,   20
    db   16,   13
    db   26,   17


    db   25,  -27
    db   33,   -1
    db    6,   11
    db  -39,   21
    db   -4,    8
    db  -42,   17
    db   25,   -6
    db   15,  -27


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

