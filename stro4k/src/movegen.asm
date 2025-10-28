section .text
default rel

; rdi - buffer (allows the use of stosw)
; rsi - position
gen_moves:
    push rbx
    push rbp
    movzx eax, byte [rsi + Board.side_to_move]

    ; r9 - occ
    mov r9, qword [rsi + Board.colors]
    or r9, qword [rsi + Board.colors + 8]

    ; r10 - side
    mov r10, qword [rsi + Board.colors + rax * 8]

    ; r11 - pieces
    imul eax, eax, 48
    lea r11, [rsi + rax] ; pieces

    lea rcx, [move_fns + 8]
    mov ebx, 5
.piece_moves_head:
    mov r12, qword [r11 + 8 * rbx]
.gen_piece_head:
    blsi r8, r12
    jz .gen_piece_end
    xor r12, r8

    call rcx

    andn rbp, r10, rax
    tzcnt rdx, r8
.serialise_head:
    tzcnt rax, rbp
    jc .serialise_end
    ; jc .gen_piece_head
    btr rbp, rax

    bt r9, rax
    jnc .serialise_no_capture

    mov ah, CAPTURE_FLAG >> 2
.serialise_no_capture:
    shl eax, 6
    ; or eax, edx
    add eax, edx
    stosd

    jmp .serialise_head
.serialise_end:
    jmp .gen_piece_head

.gen_piece_end:
    sub rcx, 2
    dec ebx
    jnz .piece_moves_head

    ; rax - pawns
    mov rax, qword [r11]

    ; r8 - enemy
    mov r8, r9
    xor r8, r10

    cmp byte [rsi + Board.side_to_move], 0
    je .gen_white_pawn_start

    bswap r8
    bswap r9
    bswap rax
.gen_white_pawn_start:
    mov rdx, rax
    shl rdx, 8
    andn rdx, r9, rdx
    push rdx

    and edx, 00FF_0000h
    shl rdx, 8
    andn rdx, r9, rdx
    push rdx

    ; pawn attacks
    mov rdx, rax
    shl rdx, 9
    and rdx, qword [NOT_A_FILE]
    push rdx

    mov rdx, rax
    shl rdx, 7
    and rdx, qword [NOT_H_FILE]
    push rdx

    xor ecx, ecx
    movzx edx, byte [rsi + Board.ep]
    cmp byte [rsi + Board.side_to_move], 0
    je .gen_white_pawn_end

    ; use xors to flip vertically
    mov ecx, 7070o
    xor edx, 70o
.gen_white_pawn_end:

    ; this is fine even after the xor
    cmp edx, 64
    jge .no_ep

    mov rax, qword [rsp]
    bt rax, rdx
    jnc .no_ep_1

    mov eax, edx
    shl eax, 6
    lea eax, [rax + rdx + (EN_PASSANT_FLAG << 12) - 7]
    xor eax, ecx
    stosd
.no_ep_1:
    mov rax, qword [rsp + 8]
    bt rax, rdx
    jnc .no_ep_2

    mov eax, edx
    shl eax, 6
    lea eax, [rax + rdx + (EN_PASSANT_FLAG << 12) - 9]
    xor eax, ecx
    stosd
.no_ep_2:
.no_ep:

    ; attacks
    and qword [rsp], r8
    and qword [rsp + 8], r8

    mov ebx, 0810_0907h
.pawn_serialise_outer_head:
    pop rbp

    ; preserve move generation order
    ; test ecx, ecx
    cmp byte [rsi + Board.side_to_move], 0
    jz .pawn_serialise_white
    bswap rbp
.pawn_serialise_white:
.pawn_serialise_inner_head:
    tzcnt rax, rbp ; index
    jc .pawn_serialise_inner_end
    btr rbp, rax

    xor al, cl

    mov edx, eax
    shl eax, 6
    ; or eax, edx
    add eax, edx
    sub al, bl

    xor eax, ecx

    ; capture
    test bl, 1
    jz .pawn_serialise_no_capture

    or ah, CAPTURE_FLAG << 4
.pawn_serialise_no_capture:

    test bl, 10h
    jz .pawn_serialise_no_double_pawn_push
    or ah, DOUBLE_PAWN_PUSH_FLAG << 4
    ; or ah, bl
.pawn_serialise_no_double_pawn_push:

    ; promo
    cmp edx, 56
    jnae .pawn_serialise_no_promo
    or ah, (PROMO_FLAG << 4) | 30h

    stosd
    sub ah, 10h
    stosd
    sub ah, 10h
    stosd
    sub ah, 10h
.pawn_serialise_no_promo:
    stosd

    jmp .pawn_serialise_inner_head
.pawn_serialise_inner_end:
    shr ebx, 8
    jnz .pawn_serialise_outer_head

    ; castling
    movzx edx, byte [rsi + Board.castling]
    cmp byte [rsi + Board.side_to_move], 0
    jz .white_castling

    shr edx, 2
.white_castling:
    test edx, 1
    jz .no_queenside_castling

    test r9b, 0b0000_1110
    jnz .no_queenside_castling

    mov eax, (QUEENSIDE_CASTLE_FLAG << 12) | (2 << 6) | 4
    xor eax, ecx
    stosd

.no_queenside_castling:
    test edx, 2
    jz .no_kingside_castling

    test r9b, 0b0110_0000
    jnz .no_kingside_castling

    mov eax, (KINGSIDE_CASTLE_FLAG << 12) | (6 << 6) | 4
    xor eax, ecx
    stosd
.no_kingside_castling:

    pop rbp
    pop rbx
    ret


; r8 - gen (preserved)
; r9 - occ (preserved)
rook_moves:
    vmovdqu xmm7, oword [ALL_MASK]
    vpmovzxbq xmm1, word [ROOK_SHIFTS]
    jmp dumb7fill

; r8 - gen (preserved)
; r9 - occ (preserved)
bishop_moves:
    vmovdqu xmm7, oword [NOT_A_FILE]
    vpmovzxbq xmm1, word [BISHOP_SHIFTS]
    
    ; jmp dumb7fill

; r8 - gen (preserved)
; r9 - occ (preserved)
; l_mask - xmm7,
; shifts - xmm1
dumb7fill:
%ifdef AVX512
    vpbroadcastq xmm2, r8 ; l_gen
    vpbroadcastq xmm4, r9 ; occ

    vmovdqu xmm3, xmm2 ; r_gen

    mov al, 7
.loop_head:
    vpand xmm6, xmm3, xmm7
    vpsllvq xmm5, xmm2, xmm1
    vpand xmm5, xmm5, xmm7
    vpsrlvq xmm6, xmm6, xmm1

    vpternlogq xmm2, xmm4, xmm5, 0F2h
    vpternlogq xmm3, xmm4, xmm6, 0F2h

    dec al
    jnz .loop_head

%else
    vmovq xmm2, r8 ; gen
    vmovq xmm4, r9, ; gen
    vpbroadcastq xmm2, xmm2 ; l_gen
    vpbroadcastq xmm4, xmm4 ; occ

    vmovdqu xmm3, xmm2 ; r_gen

    mov al, 7
    jmp .loop_start

.loop_head:
    vpandn xmm5, xmm4, xmm5
    vpandn xmm6, xmm4, xmm6
    vpor xmm2, xmm2, xmm5
    vpor xmm3, xmm3, xmm6
.loop_start:
    vpsllvq xmm5, xmm2, xmm1
    vpand xmm6, xmm3, xmm7
    vpand xmm5, xmm5, xmm7
    vpsrlvq xmm6, xmm6, xmm1

    dec al
    jnz .loop_head
%endif
.or:
    vpor xmm7, xmm5, xmm6
    vpunpckhqdq xmm1, xmm7, xmm7

    vpor xmm1, xmm7, xmm1
    vmovq rax, xmm1
    ret

move_fns:
move_fn_knight:
    jmp knight_moves
    %if $ - move_fn_knight != 2
    %error "knight jump not short"
    %endif

move_fn_bishop:
    jmp bishop_moves
    %if $ - move_fn_bishop != 2
    %error "bishop jump not short"
    %endif

move_fn_rook:
    jmp rook_moves
    %if $ - move_fn_rook != 2
    %error "rook jump not short"
    %endif

move_fn_queen:
    jmp queen_moves
    %if $ - move_fn_queen != 2
    %error "queen jump not short"
    %endif

move_fn_king:
    jmp king_moves
    %if $ - move_fn_king != 2
    %error "king jump not short"
    %endif

; r8 - gen (preserved)
; r9 - occ (preserved)
queen_moves:
    call rook_moves
    xchg rax, rdx

    call bishop_moves
    or rax, rdx
    ret

; r8 - gen (preserved)
; r9 - preserved
knight_moves:
    vmovdqu ymm7, yword [NOT_A_FILE]
    vpmovzxbq ymm1, dword [KNIGHT_SHIFTS]

%ifdef AVX512
    vpbroadcastq ymm2, r8

    vpsllvq ymm5, ymm2, ymm1
    vpand ymm4, ymm2, ymm7
    vpsrlvq ymm4, ymm4, ymm1

    vpternlogq ymm5, ymm4, ymm7, 0ECh
%else
    vmovq xmm2, r8
    vpbroadcastq ymm2, xmm2
    
    vpsllvq ymm3, ymm2, ymm1
    vpand ymm4, ymm2, ymm7
    vpand ymm3, ymm3, ymm7
    vpsrlvq ymm4, ymm4, ymm1

    vpor ymm5, ymm4, ymm3
%endif
    vextracti128 xmm6, ymm5, 1
    jmp dumb7fill.or

; r8 - gen (preserved)
; r9 - preserved
king_moves:
    mov rax, r8
    shr rax, 1
    and rax, qword [NOT_H_FILE]

    lea rdx, [r8 + r8]
    and rdx, qword [NOT_A_FILE]

    or rax, r8
    or rax, rdx

    mov rdx, rax
    shl rdx, 8
    or rax, rdx

    mov rdx, rax
    shr rdx, 8
    or rax, rdx

    ret
    

