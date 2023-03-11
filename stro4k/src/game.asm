default rel
section .text

%ifdef EXPORT_SYSV
global board_is_area_attacked_sysv
global board_hash
global game_is_repetition_sysv
global game_make_move_sysv

board_is_area_attacked_sysv:
    xchg rdi, rsi
    jmp board_is_area_attacked

game_is_repetition_sysv:
    push rbx
    mov rbx, rdi
    call game_is_repetition
    pop rbx
    ret

game_make_move_sysv:
    push rbx
    mov rbx, rdi
    mov rdx, rsi
    call game_make_move
    pop rbx
    ret

%endif
; rbx - game
; rdx - move
game_make_move:

    ; make copy of board
    mov ecx, Board_size
    mov rsi, qword [rbx] ; rsi old board
    lea rdi, [rsi + rcx]

    rep movsb ; rsi - new board

    mov al, 64
    test dx, dx
    jz .null_move

    push rbx
    push rdx
    ; eax - side to move
    movzx eax, byte [rsi + Board.side_to_move]

    ; multiply by 48
    shl eax, 4
    lea eax, [rax + 2 * rax]

    ; pieces - r8
    lea r8, qword [rsi + Board.pieces + rax]

    ; enemy - r9
    xor al, 48
    lea r9, qword [rsi + Board.pieces + rax]

    ; ebx - flags
    mov ebx, edx
    shr ebx, 12

    ; dh - dest, dl - origin
    mov edi, 3F3Fh
    pdep edx, edx, edi
    ; push rdx ; save move

    ; eax - piece
    call board_get_piece

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

    ; switch origin and dest
    xchg dl, dh

    ; rdi - destination mask
    xor edi, edi
    bts rdi, rdx

    test bl, PROMO_FLAG
    jz .no_promo
    
    ; promo piece
    mov eax, ebx
    and al, 11b
    inc al
.no_promo:
    ; update pieces
    xor qword [r8 + 8 * rax], rdi

    ; rdi king area
    mov rdi, qword [r8 + 40]

    cmp al, 5 ; king can't be promo piece, so al has not changed
    jne .no_king_move

    ; rbx - castle rook mask
    mov ecx, 00001001b
    mov eax, 00011100b
    cmp bl, QUEENSIDE_CASTLE_FLAG
    je .castle
    mov cl, 10100000b
    mov al, 01110000b
    cmp bl, KINGSIDE_CASTLE_FLAG
    je .castle
    jmp .no_castle

.castle:
    cmp byte [rsi + Board.side_to_move], 0
    je .white_castle

    bswap rcx
    bswap rax
.white_castle:
    xor qword [r8 + 24], rcx
    mov rdi, rax ; update king area
.no_castle:
    mov cl, 1100b ; upper part of rcx is irrelevant
    cmp byte [rsi + Board.side_to_move], 0
    je .white_king_move
    shr ecx, 2
.white_king_move:
    and byte [rsi + Board.castling], cl
.no_king_move:
    test bl, CAPTURE_FLAG
    jz .no_capture

    cmp bl, EN_PASSANT_FLAG
    jne .no_ep
    ; en passant

    ; edx - captured index
    mov eax, 3807h ; rank of origin, file of dest
    pext edx, edx, eax
    xor eax, eax ; 0 = pawn
    jmp .remove_captured
.no_ep:
    mov r8, r9 ; r8 - enemy pieces
    call board_get_piece
.remove_captured:
    xor ebx, ebx
    bts rbx, rdx
    xor qword [r9 + 8 * rax], rbx
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
    call board_is_area_attacked
    pop rdx ; restore move
    pop rbx ; restore game
    ; test al, al
    ; jnz .end
    xor al, 1
    jz .end

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

    setnc al ; al = 1

    mov qword [rbx], rsi ; update Game
.end:
    ret

; pieces - r8
; square - rdx
board_get_piece:
    mov eax, 5
.loop_head:
    mov rcx, qword [r8 + 8 * rax]
    bt rcx, rdx
    jc .end
    dec eax
    jns .loop_head

.end:
    ret
    

; board - rsi
; area - rdi
board_is_area_attacked:
    ; eax - enemy color
    movzx eax, byte [rsi + Board.side_to_move]
    xor al, 1
    shl eax, 4
    lea eax, [rax + 2 * rax] ; multiply by 48

    ; r10 - enemy pieces
    lea r10, [rsi + Board.pieces + rax]

    ; r8 - enemy pawns
    mov r8, qword [r10]

    test al, al
    jz .enemy_white

    ; exchange attacking pawns and area for black
    xchg r8, rdi 
.enemy_white:
    mov r9, qword [NOT_A_FILE]
    mov rcx, r8
    mov rdx, r8
    shl rcx, 9
    and rcx, r9
    and rdx, r9
    shl rdx, 7
    or rcx, rdx

    test rdi, rcx
    jnz .end

    ; area - r8
    test al, al
    cmovz r8, rdi ; enemy is white

    ; r9 - occ
    mov r9, qword [rsi + Board.white]
    or r9, qword [rsi + Board.black]

    
    ; possible performance improvement from iterating tho other way
    lea rcx, qword [move_fns + 8]
    mov edi, 5
.piece_moves_head:
    call rcx
    test rax, qword[r10 + 8 * rdi]
    jnz .end
    sub rcx, 2
    dec edi
    jnz .piece_moves_head
.end:
    setnz al
    ret

; rsi - board
board_hash:
    mov eax, dword [rsi + Board.side_to_move]
    and eax, 00FFFFFFh
    vmovd xmm0, eax

    mov eax, 12
.loop_head:
    vaesenc xmm0, xmm0, oword [rsi + Board.pieces + 8 * rax]
    dec eax
    jns .loop_head

    vmovq rax, xmm0
    ret
    

; game - rbx
game_is_repetition:
    ; rdi - current position
    mov rdi, qword [rbx]

    ; rsi - position to search
    mov rsi, rdi

    ; repeating positions remaining, ends at -1 so that we have
    ; ZF == 1 at the end
    mov dl, 1 
.loop_head:
    ; check for 50 move reset
    cmp byte [rsi + Board.fifty_moves], 0
    je .end ; ZF == 1

    ; previous position
    add rsi, -Board_size

    mov ecx, 115
    repe cmpsb

    ; reset rdi and rsi without affecting flags
    lea rsi, [rsi + rcx - 115]
    lea rdi, [rdi + rcx - 115]

    jne .loop_head

    dec dl
    jns .loop_head
    
    ; ZF == 0
.end:
    setnz al
    ret

