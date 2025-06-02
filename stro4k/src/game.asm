default rel
section .text

; rdx - move
; carry flag indicates illegality
; Sets rsi to a pointer to the new board
; Sets r8 to the pieces of the side that moved
; Sets r9 to the pieces of the side to move
game_make_move:
    ; make copy of board
    mov ecx, Board_size
    mov rsi, qword [rbx] ; rsi old board
    lea rdi, [rsi + rcx]

    rep movsb ; rsi - new board

    lea r8, [rsi + Board.white_pieces]
    lea r9, [rsi + Board.black_pieces]
    cmp byte [rsi + Board.side_to_move], 0
    je .white_to_move

    xchg r8, r9
.white_to_move:
    mov al, 64
    test edx, edx
    jz .null_move

    push rbx
    push rdx

    ; ebx - flags
    mov ebx, edx
    shr ebx, 12

    ; eax - origin, ecx - dest
    call board_get_move_pieces

    ; dh - dest, dl - origin
    mov edi, 3F3Fh
    pdep edx, edx, edi

    ; reset 50 move rule
    test al, al ; test for pawn
    jz .fifty_moves_reset
    test bl, CAPTURE_FLAG
    jz .no_fifty_moves_reset
.fifty_moves_reset:
    mov byte [rsi + Board.fifty_moves], -1
.no_fifty_moves_reset:

    ; remove origin
    xor edi, edi
    bts rdi, rdx
    xor qword [r8 + 8 * rax], rdi

    test bl, PROMO_FLAG
    jz .no_promo
    
    ; promo piece
    mov eax, ebx
    and al, 11b
    inc al
.no_promo:
    ; switch origin and dest
    xchg dl, dh

    ; set move index
    mov edi, eax
    shl edi, 6
    add dil, dl

    ; movzx edi, dl
    ; lea edi, [rdi + 2 * rdi]
    ; lea edi, [rax + 2 * rdi]

    mov dword [rsi + Board.move_index], edi

    ; rdi - destination mask
    xor edi, edi
    bts rdi, rdx
    xor qword [r8 + 8 * rax], rdi

    cmp al, 5 ; king can't be promo piece, so al has not changed
    jne .no_king_move

    mov al, 1100b ; upper part of rax is irrelevant
    cmp byte [rsi + Board.side_to_move], 0
    je .white_king_move
    shr eax, 2
.white_king_move:
    and byte [rsi + Board.castling], al

    ; rbx - castle rook mask
    mov eax, 00001001b
    mov edi, 00011100b
    cmp ebx, QUEENSIDE_CASTLE_FLAG
    je .castle
    mov eax, 10100000b
    mov edi, 01110000b
    cmp ebx, KINGSIDE_CASTLE_FLAG
    jne .no_castle

.castle:
    cmp byte [rsi + Board.side_to_move], 0
    je .white_castle

    bswap rax
    bswap rdi
.white_castle:
    xor qword [r8 + 24], rax
    jmp .king_move
.no_castle:
.no_king_move:
    ; rdi king area
    mov rdi, qword [r8 + 40]
.king_move:
    test bl, CAPTURE_FLAG
    jz .no_capture

    cmp bl, EN_PASSANT_FLAG
    jne .no_ep
    ; en passant

    ; edx - captured index
    mov eax, 3807h ; rank of origin, file of dest
    pext edx, edx, eax
    xor ecx, ecx ; 0 = pawn
.no_ep:
    ; remove the captured piece
    xor eax, eax
    bts rax, rdx
    xor qword [r9 + 8 * rcx], rax
.no_capture:

    xor eax, eax
    xor ebx, ebx
    mov ecx, 5
.update_colors_head:
    or rax, qword [rsi + Board.white_pieces + 8 * rcx]
    or rbx, qword [rsi + Board.black_pieces + 8 * rcx]
    dec ecx
    jns .update_colors_head

    mov qword [rsi + Board.white], rax
    mov qword [rsi + Board.black], rbx

    ; rdi - area
    call board_area_attacked_by
    pop rdx ; restore move
    pop rbx ; restore game

    ; Return with CF=1 for illegal move if edi is not 6
    cmp edi, 6
    jc .end

    ; rdi - origin
    xor edi, edi
    bts rdi, rdx
    
    ; al - origin square
    mov al, dl
    and al, 63
    shr edx, 6

    ; rdi - origin | dest
    bts rdi, rdx

    mov cl, dl
    and cl, 63
    shr edx, 6

    ; al - ep target - (origin + dest) / 2
    add al, cl
    shr al, 1

    cmp dl, DOUBLE_PAWN_PUSH_FLAG
    mov cl, 64
    cmovne eax, ecx

    ; remove castling rights 
    mov rcx, 8100_0000_0000_0081h
    pext rcx, rdi, rcx
    not cl
    and byte [rsi + Board.castling], cl

.null_move:
    ; set ep
    mov byte [rsi + Board.ep], al

    ; update 50 move rule
    inc byte [rsi + Board.fifty_moves]

    ; update side to move, clear CF
    xor byte [rsi + Board.side_to_move], 1

    ; update Game
    mov qword [rbx], rsi
.end:
    ret

; pieces - r8
; move - rdx
; returns origin in eax, dest in ecx
; clobbers rax, rcx, rdi
board_get_move_pieces:
    mov ecx, 5
.loop_head_1:
    mov rdi, qword [r8 + 8 * rcx]
    bt rdi, rdx
    jc .end_1
    dec ecx
    jns .loop_head_1
.end_1:
    xor r8, 48
    ror edx, 6
    mov eax, ecx

    mov ecx, 5
.loop_head_2:
    mov rdi, qword [r8 + 8 * rcx]
    bt rdi, rdx
    jc .end_2
    dec ecx
    jns .loop_head_2
.end_2:
    xor r8, 48
    rol edx, 6
    ret
    

; board - rsi
; area - rdi
; Returns attacker piece in edi and attacker squares in rax
; indicates that there is no attacker by edi = 6 and ZF=1
; interprets the occupied squares as white ^ black
; sets r10 to enemy pieces
board_area_attacked_by:
    ; eax - enemy color
    movzx eax, byte [rsi + Board.side_to_move]
    xor al, 1
    shl eax, 4
    lea eax, [rax + 2 * rax] ; multiply by 48

    ; r10 - enemy pieces
    lea r10, [rsi + Board.pieces + rax]

    ; r8 - area
    mov r8, rdi
    mov r9, qword [NOT_A_FILE]

    test al, al
    mov rax, rdi
    jz .enemy_white

    ; pawn moves from area
    shl rax, 9
    and rax, r9
    and rdi, r9
    shl rdi, 7
    jmp .enemy_black
.enemy_white:
    shr rax, 7
    and rax, r9
    and rdi, r9
    shr rdi, 9
.enemy_black:
    or rax, rdi

    ; pawn attacks
    xor edi, edi
    and rax, qword [r10]
    jnz .end

    ; r9 - occ
    ; Usually, white and black pieces are disjoint and xor is equivalent to or,
    ; however using xor here allows a simpler SEE implementation
    mov r9, qword [rsi + Board.white]
    xor r9, qword [rsi + Board.black]
    
    lea rcx, [move_fns]
    mov edi, 1
.piece_moves_head:
    call rcx
    and rax, qword [r10 + 8 * rdi]
    jnz .end
    add rcx, 2
    inc edi
    cmp edi, 6
    jne .piece_moves_head ; jne = jnz
.end:
    ret

; rsi - board
; returns value in NZ flag and al
board_is_check:
    movzx eax, byte [rsi + Board.side_to_move]
    shl eax, 4
    lea eax, [rax + 2 * rax] ; multiply by 48

    mov rdi, qword [rsi + rax + 40] ; king

    call board_area_attacked_by
    setnz al
    ret

