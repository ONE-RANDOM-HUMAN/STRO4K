//! Move generation:
//! 850 bytes allocated for binary
//!

use crate::consts::{AB_FILE, ALL, A_FILE, H_FILE};
use crate::position::{Bitboard, Board, Color, Move, MoveFlags, MovePlus, Square};

pub type MoveFn = fn(Bitboard, Bitboard) -> Bitboard;
pub type MoveBuf = std::mem::MaybeUninit<[MovePlus; 256]>;

pub fn gen_moves<'a>(position: &Board, buf: &'a mut MoveBuf) -> &'a mut [MovePlus] {
    let start: *mut MovePlus = buf.as_mut_ptr().cast();
    let mut ptr = start;

    {
        let pieces = position.pieces()[position.side_to_move() as usize];
        let occ = position.white() | position.black();
        let side = position.colors()[position.side_to_move() as usize];

        // generate all non pawn, non castling moves

        // This is how it would be done in asm, so it is done this way here
        // so that the behaviour is consistent.
        let movements = [
            knight_moves,
            bishop_moves,
            rook_moves,
            queen_moves,
            king_moves,
        ];

        // iterate from 5 to 1 (inclusive)
        let mut i = 5;
        while i != 0 {
            // SAFETY: There is sufficient memory in `buf` to store the moves
            ptr = unsafe { gen_piece(ptr, pieces[i], occ, side, movements[i - 1]) };

            i -= 1;
        }

        // SAFETY: There is sufficient memory in `buf` to store the moves
        unsafe {
            ptr = gen_pawn(ptr, position, pieces[0], occ, occ ^ side);
            ptr = gen_castle(ptr, position, occ);
        }
    }

    // // SAFETY: begin..ptr is a valid pointer range in buf
    unsafe { std::slice::from_raw_parts_mut(start, ptr.offset_from(start) as usize) }
}

pub(super) fn dumb7fill(gen: Bitboard, l_mask: Bitboard, occ: Bitboard, shift: u32) -> Bitboard {
    let (mut l_gen, mut r_gen) = (gen, gen);

    // only 6 required for attacks
    for _ in 0..6 {
        l_gen |= (l_gen << shift) & l_mask & !occ;
        r_gen |= ((r_gen & l_mask) >> shift) & !occ;
    }

    ((l_gen << shift) & l_mask) | ((r_gen & l_mask) >> shift)
}

pub fn knight_moves(pieces: Bitboard, _occ: Bitboard) -> Bitboard {
    let out_1 = ((pieces << 1) & !A_FILE) | ((pieces & !A_FILE) >> 1);
    let out_2 = ((pieces << 2) & !AB_FILE) | ((pieces & !AB_FILE) >> 2);
    (out_1 << 16) | (out_1 >> 16) | (out_2 << 8) | (out_2 >> 8)
}

pub fn bishop_moves(pieces: Bitboard, occ: Bitboard) -> Bitboard {
    dumb7fill(pieces, !A_FILE, occ, 9) | dumb7fill(pieces, !H_FILE, occ, 7)
}

pub fn rook_moves(pieces: Bitboard, occ: Bitboard) -> Bitboard {
    dumb7fill(pieces, !A_FILE, occ, 1) | dumb7fill(pieces, ALL, occ, 8)
}

pub fn queen_moves(pieces: Bitboard, occ: Bitboard) -> Bitboard {
    bishop_moves(pieces, occ) | rook_moves(pieces, occ)
}

pub fn king_moves(pieces: Bitboard, _occ: Bitboard) -> Bitboard {
    let rank = pieces | ((pieces << 1) & !A_FILE) | ((pieces & !A_FILE) >> 1);
    rank | (rank << 8) | (rank >> 8)
}

/// # Safety
/// `ptr` must be valid for a sufficient number of writes.
pub(super) unsafe fn gen_piece(
    mut ptr: *mut MovePlus,
    mut pieces: Bitboard,
    occ: Bitboard,
    side: Bitboard,
    movement: MoveFn,
) -> *mut MovePlus {
    while pieces != 0 {
        let square = pieces & pieces.wrapping_neg();
        let dests = movement(square, occ) & !side;

        let square = Square::from_index(square.trailing_zeros() as u8).unwrap();

        // SAFETY: The ptr is valid by the safety requirements of the function
        ptr = unsafe { serialise(ptr, square, dests, occ) };

        pieces &= pieces - 1;
    }

    ptr
}

/// # Safety
/// `ptr` must be valid for a sufficient number of writes.
pub(super) unsafe fn gen_pawn(
    mut ptr: *mut MovePlus,
    position: &Board,
    pawns: Bitboard,
    occ: Bitboard,
    enemy: Bitboard,
) -> *mut MovePlus {
    let consts: [i8; 4] = if position.side_to_move() == Color::White {
        [8, 16, 9, 7]
    } else {
        [-8, 40, -7, -9]
    };

    let single_pushes = pawns.rotate_left(consts[0] as u32 & 63) & !occ;
    let double_pushes =
        (single_pushes & 0xFF << consts[1]).rotate_left(consts[0] as u32 & 63) & !occ;

    let kingside_attacks = pawns.rotate_left(consts[2] as u32 & 63) & !A_FILE;
    let queenside_attacks = pawns.rotate_left(consts[3] as u32 & 63) & !H_FILE;

    if let Some(target) = position.ep() {
        if target.intersects(queenside_attacks) {
            // SAFETY: The ptr is valid by the safety requirements of the function
            unsafe {
                ptr.write(
                    Move::new(
                        target.offset(-consts[3]).unwrap(),
                        target,
                        MoveFlags::EN_PASSANT,
                    )
                    .into(),
                );
                ptr = ptr.add(1);
            }
        }

        if target.intersects(kingside_attacks) {
            // SAFETY: The ptr is valid by the safety requirements of the function
            unsafe {
                ptr.write(
                    Move::new(
                        target.offset(-consts[2]).unwrap(),
                        target,
                        MoveFlags::EN_PASSANT,
                    )
                    .into(),
                );
                ptr = ptr.add(1);
            }
        }
    }

    // SAFETY: The ptr is valid by the safety requirements of the function
    unsafe {
        ptr = pawn_serialise(
            ptr,
            queenside_attacks & enemy,
            consts[3],
            MoveFlags::CAPTURE,
        );
        ptr = pawn_serialise(ptr, kingside_attacks & enemy, consts[2], MoveFlags::CAPTURE);
        ptr = pawn_serialise(
            ptr,
            double_pushes,
            2 * consts[0],
            MoveFlags::DOUBLE_PAWN_PUSH,
        );
        ptr = pawn_serialise(ptr, single_pushes, consts[0], MoveFlags::NONE);
    }

    ptr
}

/// # Safety
/// `ptr` must be valid for a sufficient number of writes.
pub(super) unsafe fn gen_castle(
    mut ptr: *mut MovePlus,
    position: &Board,
    occ: Bitboard,
) -> *mut MovePlus {
    let (castle, occ, origin) = if position.side_to_move() == Color::White {
        (position.castling(), occ, Square::E1)
    } else {
        (position.castling() >> 2, occ >> 56, Square::E8)
    };

    // queenside castle
    if castle & 0b1 != 0 && occ & 0b0000_1110 == 0 {
        // SAFETY: The ptr is valid by the safety requirements of the function
        unsafe {
            ptr.write(
                Move::new(
                    origin,
                    origin.offset(-2).unwrap(),
                    MoveFlags::QUEENSIDE_CASTLE,
                )
                .into(),
            );
            ptr = ptr.add(1);
        }
    }

    if castle & 0b10 != 0 && occ & 0b0110_0000 == 0 {
        // SAFETY: The ptr is valid by the safety requirements of the function
        unsafe {
            ptr.write(
                Move::new(
                    origin,
                    origin.offset(2).unwrap(),
                    MoveFlags::KINGSIDE_CASTLE,
                )
                .into(),
            );
            ptr = ptr.add(1);
        }
    }

    ptr
}

/// # Safety
/// `ptr` must be valid for a sufficient number of writes.
pub(super) unsafe fn serialise(
    mut ptr: *mut MovePlus,
    origin: Square,
    mut dests: Bitboard,
    enemy: Bitboard,
) -> *mut MovePlus {
    while dests != 0 {
        let dest = Square::from_index(dests.trailing_zeros() as u8).unwrap();

        // SAFETY: The ptr is valid by the safety requirements of the function
        unsafe {
            ptr.write(
                Move::new(
                    origin,
                    dest,
                    if dest.intersects(enemy) {
                        MoveFlags::CAPTURE
                    } else {
                        MoveFlags::NONE
                    },
                )
                .into(),
            );
            ptr = ptr.add(1);
        }

        dests &= dests - 1
    }

    ptr
}

/// # Safety
/// `ptr` must be valid for a sufficient number of writes.
pub(super) unsafe fn pawn_serialise(
    mut ptr: *mut MovePlus,
    mut squares: Bitboard,
    offset: i8,
    flags: MoveFlags,
) -> *mut MovePlus {
    while squares != 0 {
        let index = squares.trailing_zeros() as u8;
        let dest = Square::from_index(index).unwrap();
        let origin = dest.offset(-offset).unwrap();

        // promo
        if !(8..56).contains(&index) {
            // would be implemented differently in binary
            for i in (0..4).rev() {
                // add promo piece
                let flags = MoveFlags(flags.0 | MoveFlags::PROMO.0 | i);

                // SAFETY: The ptr is valid by the safety requirements of the function
                unsafe {
                    ptr.write(Move::new(origin, dest, flags).into());
                    ptr = ptr.add(1);
                }
            }
        } else {
            // SAFETY: The ptr is valid by the safety requirements of the function
            unsafe {
                ptr.write(Move::new(origin, dest, flags).into());
                ptr = ptr.add(1);
            }
        }

        squares &= squares - 1;
    }

    ptr
}
