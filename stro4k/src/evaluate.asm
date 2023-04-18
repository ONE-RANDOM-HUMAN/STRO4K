MAX_EVAL equ 128 * 256 - 1
MIN_EVAL equ -MAX_EVAL

MG_BISHOP_PAIR equ 80
EG_BISHOP_PAIR equ 178

MG_OPEN_FILE equ 103
EG_OPEN_FILE equ 4
MG_SEMI_OPEN_FILE equ 56
EG_SEMI_OPEN_FILE equ 0

section .rodata
EVAL_WEIGHTS:
MATERIAL_EVAL:
    dw 259, 280
    dw 782 - 4 * 26, 659 - 4 * 25
    dw 890 - 6 * 29, 685 - 6 * 9
    dw 1123 - 7 * 26, 1104 - 7 * 1
    dw 2539 - 13 * 20, 2115 - 13 * 0

MOBILITY_EVAL:
    db 26, 25
    db 29,  9
    db 26,  1
    db 20,  0


; doubled and isolated pawn eval
; first two in each row are isolated mg and eg
; second two are doubled mg and eg
DOUBLED_ISOLATED_PAWN_EVAL:
    db 18, 20, 67, 52
    db 27, 12, 48, 32
    db 38, 25, 65, 24
    db 68, 37, 45, 20
    db 94, 33, 44, 24
    db 43, 30, 44, 42
    db 42, 29, 23, 42
    db 95, 24, 29, 44

; in reverse order because lzcnt is used
PASSED_PAWN_EVAL:
    db 107, 235
    db 125, 144
    db  45,  76
    db   0,  23
    db   0,   0
    db   0,   0

default rel
section .text

; board - rsi
; preserves rbx, rbp, r14, r15
evaluate:
    push rbx
    push rbp
    lea rbp, [EVAL_WEIGHTS]
    mov r10, rsi
    lea r11, [rsi + Board.black_pieces]
    mov r13, qword [NOT_A_FILE]

    ; r9 - occ
.side_eval_head:
    mov r9, qword [rsi + Board.white]
    or r9, qword [rsi + Board.black]

    mov ecx, 4
    xor ebx, ebx
.material_eval_head:
    popcnt rax, qword [r10 + 8 * rcx]

    ; SWAR multiplication for MG and EG eval
    ; since it must be positive
    imul eax, dword [rbp + 4 * rcx]
    add ebx, eax

    dec ecx
    jns .material_eval_head

    ; bishop pair
    mov rcx, LIGHT_SQUARES
    mov rax, qword [r10 + 16]
    test rax, rcx
    jz .no_bishop_pair
    not rcx
    test rax, rcx
    jz .no_bishop_pair

    add ebx, MG_BISHOP_PAIR + (EG_BISHOP_PAIR << 16)
.no_bishop_pair:

    ; mobility
    mov edi, 4

    ; rsi - move fns
    lea rsi, qword [move_fns + 6]
.mobility_head:
    ; r12 - piece
    mov r12, qword [r10 + 8 * rdi]

.mobility_piece_head:
    blsi r8, r12
    jz .mobility_end_piece
    xor r12, r8

    ; mask
    mov rdx, qword [r11] ; enemy pawns
    lea rax, [rdx + rdx]
    and rax, r13
    and rdx, r13
    shr rdx, 1
    or rax, rdx

    ; Based on alignment, 8 if white to move, 56 otherwise
    lea ecx, [r10 + 8]
    ror rax, cl
    xchg rax, rcx
    not rcx

    call rsi

    and rax, rcx

    popcnt rax, rax

    ; EG << 16 + MG
    movzx edx, word [rbp + MOBILITY_EVAL - EVAL_WEIGHTS + 2 * rdi - 2]
    shl edx, 8
    xchg dl, dh

    imul eax, edx
    add ebx, eax
    jmp .mobility_piece_head
.mobility_end_piece:
    sub rsi, 2
    dec edi
    jnz .mobility_head

    mov r12, r13
    not r12

    ; doubled and isolated pawns and open file
    ; r9 - file
    mov r9, r12
    xor ecx, ecx ; loop counter
.doubled_pawns_head:
    mov r8, qword [r10] ; side pawns
    and r8, r9
    jnz .no_semi_open_file

    ; check if the file is fully open
    mov edx, MG_SEMI_OPEN_FILE + (EG_SEMI_OPEN_FILE << 16)
    mov eax, MG_OPEN_FILE + (EG_OPEN_FILE << 16)
    test r9, qword [r11] ; enemy pawns
    cmovz edx, eax

    ; find number of rooks
    mov rax, qword [r10 + 24] ; side rooks
    and rax, r9
    popcnt rax, rax
    imul eax, edx
    add ebx, eax
.no_semi_open_file:
    ; isolated pawns
    ; rax - adjacent files
    mov rdx, r9
    lea rax, [rdx + rdx]
    and rax, r13
    and rdx, r13
    shr rdx, 1
    or rax, rdx

    ; rdx - number of pawns on file
    popcnt rdx, r8

    ; load isolated and doubled pawns and SWAR-multiply by rdx
    vpmovzxbw xmm0, qword [rbp + DOUBLED_ISOLATED_PAWN_EVAL - EVAL_WEIGHTS + rcx * 4]
    vmovq rdi, xmm0
    mulx rdx, rsi, rdi ; rdx is implicit source

    test rax, qword [r10]
    jnz .no_isolated_pawns

    ; these subtractions cannot overlow because the penalty for doubled
    ; and isolated pawns is less than the value of a pawn
    sub ebx, esi
.no_isolated_pawns:
    sub rsi, rdi
    jc .no_doubled_pawns

    shr rsi, 32
    sub ebx, esi
.no_doubled_pawns:
    inc ecx
    shl r9, 1
    jnc .doubled_pawns_head

    ; add up mg and eg
    movzx eax, bx
    shr ebx, 16

    ; switch white and black
    xchg r10, r11
    cmp r10, r11

    push rbx ; eg
    push rax ; mg
    mov rsi, r11
    ja .side_eval_head


    ; passed pawns
    mov r8, qword [r10] ; white pawns
    mov r9, qword [r11] ; black pawns
    xor r11d, r11d ; loop counter
.white_passed_pawn_head:
    ; get the black pawn attack spans
    ; The leftmost bit triggers the carry flag so that the shifts
    ; are 8, 16, 32
    mov ecx, 20000008h
    mov rax, r9
.passed_pawn_south_head:
    shrx rdx, rax, rcx
    or rax, rdx
    shl ecx, 1
    jnc .passed_pawn_south_head

    ; attack spans
    mov rcx, rax
    mov rdx, rax

    shr rcx, 7
    and rcx, r13
    and rdx, r13
    shr rdx, 9

    or rax, rdx
    or rax, rcx

    ; rax - passed pawns
    andn rax, rax, r8

    mov rcx, r12
    xor esi, esi ; mg eval
    xor edi, edi ; eg eval
.passed_pawn_files_head:
    mov rdx, rax
    and rdx, rcx ; passed pawns on file
    jz .no_passed_pawn
    lzcnt rdx, rdx
    shr edx, 3

    movzx ebx, word [rbp + PASSED_PAWN_EVAL - EVAL_WEIGHTS - 2 + 2 * rdx]
    movzx edx, bl
    shr ebx, 8

    add esi, edx
    add edi, ebx
.no_passed_pawn:
    shl rcx, 1
    jnc .passed_pawn_files_head

    ; swap white and black
    xchg r9, r8
    bswap r8
    bswap r9
    dec r11d
    jpo .white_passed_pawn_end

    push rdi ; eg
    push rsi ; mg
    jmp .white_passed_pawn_head
.white_passed_pawn_end:
    ; add up all eval terms
    pop rax
    pop rbx
    sub eax, esi
    sub ebx, edi

    ; black eval
    pop rsi
    pop rdi
    sub eax, esi
    sub ebx, edi
    
    ; white eval
    pop rsi
    pop rdi
    add eax, esi
    add ebx, edi

    ; calculate phase
    mov ecx, 4
.phase_head:
    mov rsi, qword [r10 + 8 * rcx]
    or rsi, qword [r10 + 8 * rcx + 48]
    popcnt rdi, rsi
    push rdi
    
    dec ecx
    jnz .phase_head

    pop rdi
    pop rsi
    pop rcx
    pop rdx
    add edi, esi
    lea ecx, [rcx + 2 * rdx]
    lea ecx, [rdi + 2 * rcx]

    ; mg eval
    imul eax, ecx

    ; eg eval
    mov dl, 24 ; top half is zero from phase calculation
    sub edx, ecx
    imul ebx, edx

    ; divide by 24
    add ebx, eax
    movsx rax, ebx
    imul rax, rax, 2aaaaaabh
    mov rcx, rax
    sar rax, 34
    shr rcx, 63
    add eax, ecx

    ; return side to move relative eval
    test byte [r10 + Board.side_to_move], 1
    jz .white_to_move
    neg eax
.white_to_move:

    pop rbp
    pop rbx
    ret

