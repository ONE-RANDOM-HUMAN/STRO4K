section .text
default rel

%ifdef EXPORT_SYSV
global gen_moves_sysv
global knight_moves
global bishop_moves
global rook_moves
global queen_moves
global king_moves
gen_moves_sysv:
    xchg rdi, rsi
    call gen_moves

    mov rax, rdi
    ret
%endif

; rdi - buffer (allows the use of stosw)
; rsi - position
gen_moves:
    push rbp
    push rbx

    ; TODO - stop using rsi
    push rsi

    movzx eax, byte [rsi + Board.side_to_move] ; side to move

    ; rcx - side
    ; r9 - occ
    xor al, 1
    mov rcx, qword [rsi + Board.colors + rax * 8] ; enemy
    mov r9, rcx
    xor al, 1
    mov rcx, qword [rsi + Board.colors + rax * 8] ; side
    or r9, rcx

    shl eax, 4 ; multiply by 16
    lea eax, [rax + rax * 2] ; multiply by 3
    lea r10, [rsi + rax] ; pieces

    ; non pawn, non castling moves
    mov ebx, 5
    lea r11, [move_fns + 8]
.piece_moves:
    ; TODO - stop using rsi
    mov rsi, qword [r10 + 8 * rbx] ; rsi - pieces

.gen_piece_loop:
    blsi r8, rsi
    jz .gen_piece_end
    xor rsi, r8 ; remove piece

    ; r8 - pieces
    ; r9 - occ
    call r11
    andn rdx, rcx, rax

    ; rax - index of piece
    tzcnt rax, r8

    ; serialise
.serialise_loop:
    ; rbp - destination index
    tzcnt rbp, rdx
    jc .serialise_end
    btr rdx, rbp ; clear piece

    ; origin
    and eax, 63

    bt r9, rbp
    jnc .serialise_no_capture
    bts eax, 14 ; capture flag
.serialise_no_capture:
    ; destination square
    shl ebp, 6
    or eax, ebp
    stosw
    jmp .serialise_loop
.serialise_end:
    jmp .gen_piece_loop
.gen_piece_end:
    sub r11, 2
    dec ebx
    jnz .piece_moves

    ; pawns
    ; side pawns - rbx
    ; r10 points to the start of the pieces, which is the pawns
    mov rbx, qword [r10]

    ; r8 - enemy
    mov r8, r9
    xor r8, rcx

    pop rsi ; board
    mov dl, byte [rsi + Board.side_to_move]
    test dl, dl

    ; black pawn moves are generated by mirroring
    jz .gen_pawn_white_start

    ; these are the only ones that are used
    bswap r9 ; occ
    bswap rbx ; pawns
.gen_pawn_white_start:
    ; single and double pushes
    mov rax, rbx
    shl rax, 8
    andn rax, r9, rax
    push rax

    ; double pushes must end on the fourth rank,
    ; and this fits in eax
    shl eax, 8 ; second push
    andn rax, r9, rax
    push rax

    mov ecx, 0709h
    lea r10, [NOT_A_FILE]
.pawn_attacks:
    shlx rax, rbx, rcx
    and rax, qword [r10]
    push rax
    add r10, 8
    shr ecx, 8
    jnz .pawn_attacks

    ; offsets for movement
    mov ebx, 0810_0907h

    test dl, dl
    jz .gen_pawn_white_end
    mov ebx, 0F8F0_F9F7h
    bswap r9

    ; rcx = 0
    mov cl, 3
.pawn_attacks_bswap:
    movbe rax, qword [rsp + 8 * rcx]
    mov qword [rsp + 8 * rcx], rax
    dec cl
    jns .pawn_attacks_bswap
.gen_pawn_white_end:
    mov dl, byte [rsi + Board.ep]
    cmp dl, 64
    jae .no_ep

    ; en passant
    xor ecx, ecx
.ep_loop_head:
    mov rbp, qword [rsp + 8 * rcx]
    bt rbp, rdx
    jnc .ep_loop_no_ep
    mov ebp, edx
    mov eax, edx
    or ebp, EN_PASSANT_FLAG << 6
    shl ebp, 6
    sub al, bl ; offset
    or eax, ebp
    stosw
.ep_loop_no_ep:
    xchg bh, bl
    inc ecx
    cmp ecx, 2
    jb .ep_loop_head
.no_ep:
    and qword [rsp], r8
    and qword [rsp + 8], r8

    ; flags
    mov ecx, 0010_4040h
.pawn_serialise_outer_head:
    pop r10
.pawn_serialise_inner_head:
    tzcnt rax, r10
    jc .pawn_serialise_end
    btr r10, rax
    
    mov edx, eax

    ; add flags - high bits are irrelevant
    lea ebp, [eax + 4 * ecx]
    shl ebp, 6
    sub al, bl
    or eax, ebp

    sub edx, 8
    cmp edx, 48
    jnae .no_promo
    or ah, PROMO_FLAG << 4
    mov dl, 30h

.promo_serialise_head:
    xor ah, dl
    stosw
    xor ah, dl
    sub dl, 10h

    ; knight promo happens after .no_promo
    jnz .promo_serialise_head
.no_promo:
    stosw
    jmp .pawn_serialise_inner_head
.pawn_serialise_end:
    shr ecx, 8
    shr ebx, 8
    jnz .pawn_serialise_outer_head

    ; castling
    mov edx, 4 ; E1
    movzx ecx, byte [rsi + Board.castling]
    cmp byte [rsi + Board.side_to_move], 0
    je .white_castling
    shr r9, 56 ; occ
    shr ecx, 2
    add edx, 56 ; E8
.white_castling:
    test cl, 1
    jz .no_queenside_castling
    test r9b, 0000_1110b
    jnz .no_queenside_castling

    ; queenside castling
    lea eax, [edx + (QUEENSIDE_CASTLE_FLAG << 6) - 2]
    shl eax, 6
    or eax, edx
    stosw
.no_queenside_castling:
    test cl, 10b
    jz .no_kingside_castling
    test r9b, 0110_0000b
    jnz .no_kingside_castling

    ; kingside castling
    lea eax, [edx + (KINGSIDE_CASTLE_FLAG << 6) + 2]
    shl eax, 6
    or eax, edx
    stosw
.no_kingside_castling:
    pop rbx
    pop rbp
    ret


; r8 - gen (preserved)
; r9 - occ (preserved)
rook_moves:
    vmovdqu xmm0, oword [ALL_MASK]
    vmovdqu xmm1, oword [ROOK_SHIFTS]
    jmp dumb7fill

; r8 - gen (preserved)
; r9 - occ (preserved)
bishop_moves:
    vmovdqu xmm0, oword [NOT_A_FILE]
    vmovdqu xmm1, oword [BISHOP_SHIFTS]
    
    ; jmp dumb7fill

; r8 - gen (preserved)
; r9 - occ (preserved)
; l_mask - xmm0,
; shifts - xmm1
dumb7fill:
    vmovq xmm2, r8 ; gen
    vmovq xmm4, r9, ; gen
    vpunpcklqdq xmm2, xmm2, xmm2 ; l_gen
    vpunpcklqdq xmm4, xmm4, xmm4 ; occ

    vmovdqa xmm3, xmm2 ; r_gen

    mov al, 7
    jmp .loop_start

.loop_head:
    vpandn xmm5, xmm4, xmm5
    vpandn xmm6, xmm4, xmm6
    vpor xmm2, xmm2, xmm5
    vpor xmm3, xmm3, xmm6
.loop_start:
    vpsllvq xmm5, xmm2, xmm1
    vpand xmm6, xmm3, xmm0
    vpand xmm5, xmm5, xmm0
    vpsrlvq xmm6, xmm6, xmm1

    dec al
    jnz .loop_head

.or:
    vpor xmm0, xmm5, xmm6
    vpunpckhqdq xmm1, xmm0, xmm0
    vpor xmm0, xmm0, xmm1
    vmovq rax, xmm0
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
    vmovdqu ymm0, yword [NOT_A_FILE]
    vmovdqu ymm1, yword [KNIGHT_SHIFTS]

    vmovq xmm2, r8
    vpbroadcastq ymm2, xmm2
    
    vpsllvq ymm3, ymm2, ymm1
    vpand ymm4, ymm2, ymm0
    vpand ymm3, ymm3, ymm0
    vpsrlvq ymm4, ymm4, ymm1

    vpor ymm5, ymm4, ymm3
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
    
