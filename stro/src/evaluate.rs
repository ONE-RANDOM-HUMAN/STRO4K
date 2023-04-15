use crate::consts;
use crate::movegen::{bishop_moves, knight_moves, queen_moves, rook_moves, MoveFn};
use crate::position::{Bitboard, Board, Color};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
struct Eval(i16, i16);

pub const MAX_EVAL: i32 = 128 * 256 - 1;
pub const MIN_EVAL: i32 = -MAX_EVAL;

// Material eval adjusted to average mobility
const MATERIAL_EVAL: [Eval; 5] = [
    Eval(331, 297),
    Eval(788, 693).accum_to(MOBILITY_EVAL[0], -4),
    Eval(885, 709).accum_to(MOBILITY_EVAL[1], -6),
    Eval(1210, 1251).accum_to(MOBILITY_EVAL[2], -7),
    Eval(2484, 2260).accum_to(MOBILITY_EVAL[3], -13),
];

const MOBILITY_EVAL: [Eval; 4] = [Eval(33, 19), Eval(21, 10), Eval(16, 2), Eval(11, 0)];

const BISHOP_PAIR_EVAL: Eval = Eval(90, 163);

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval(-84,  16),
    Eval(-39,  24),
    Eval(-45,  11),
    Eval(-25,  -7),
    Eval(-10,   2),
    Eval( 24, -14),
    Eval( 24,  -2),
    Eval(-38, -39),
];

const PST: [[Eval; 8]; 6] = [
    [
        Eval(-7, -20),
        Eval(-54, 10),
        Eval(-14, -31),
        Eval(-33, -41),
        Eval(30, 4),
        Eval(41, -30),
        Eval(38, 75),
        Eval(41, 59),
    ],
    [
        Eval(-32, -22),
        Eval(-38, -40),
        Eval(-26, -2),
        Eval(-46, -14),
        Eval(61, 23),
        Eval(35, 14),
        Eval(11, 2),
        Eval(36, 13),
    ],
    [
        Eval(20, -19),
        Eval(-39, -23),
        Eval(7, -3),
        Eval(-14, 3),
        Eval(9, 16),
        Eval(36, 5),
        Eval(-8, 2),
        Eval(-2, 4),
    ],
    [
        Eval(-69, -43),
        Eval(-16, -48),
        Eval(-41, -18),
        Eval(-32, -10),
        Eval(22, 11),
        Eval(35, 21),
        Eval(41, 27),
        Eval(55, 33),
    ],
    [
        Eval(-19, -33),
        Eval(-7, -75),
        Eval(-19, -2),
        Eval(-31, 8),
        Eval(27, 8),
        Eval(11, 44),
        Eval(9, -3),
        Eval(29, 33),
    ],
    [
        Eval(47, -62),
        Eval(-58, -39),
        Eval(-16, -16),
        Eval(-21, 7),
        Eval(14, 26),
        Eval(21, 47),
        Eval(10, 9),
        Eval(12, 26),
    ],
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval( 0,   0),
    Eval( 0,   0),
    Eval( 0,  28),
    Eval(17,  46),
    Eval(91, 104),
    Eval(80, 141),
];

const OPEN_FILE_EVAL: Eval = Eval(68, 0);

// Tuned as (54, -1), but negative values need to be avoided
const SEMI_OPEN_FILE_EVAL: Eval = Eval(30, 0);

impl Eval {
    fn accum(&mut self, eval: Eval, count: i16) {
        *self = self.accum_to(eval, count);
    }

    const fn accum_to(self, eval: Eval, count: i16) -> Eval {
        Eval(self.0 + count * eval.0, self.1 + count * eval.1)
    }
}

fn popcnt(bb: Bitboard) -> i16 {
    bb.count_ones() as i16
}

fn resolve(board: &Board, eval: Eval) -> i32 {
    let mut phase: i32 = 0;

    const WEIGHTS: [i32; 4] = [1, 1, 2, 4];

    #[allow(clippy::needless_range_loop)]
    for i in 0..4 {
        phase += WEIGHTS[i] * popcnt(board.pieces()[0][i + 1] | board.pieces()[1][i + 1]) as i32;
    }

    let score = (i32::from(eval.0) * phase + i32::from(eval.1) * (24 - phase)) / 24;
    if board.side_to_move() == Color::White {
        score
    } else {
        -score
    }
}

/// Mirrored Quarter PSTs
/// Each entry in the pst represents a 2x2 square, and the values
/// are mirrored across the D/E file
fn side_pst(pieces: &[Bitboard; 6], row_mask: u8) -> Eval {
    let mut eval = Eval(0, 0);
    for (i, mut pieces) in pieces.iter().copied().enumerate() {
        while pieces != 0 {
            let piece_index = pieces.trailing_zeros();
            let row = (piece_index / 16) ^ row_mask as u32;
            let column = ((piece_index / 2) & 0b11).count_ones() & 0b1;

            eval.accum(PST[i][(2 * row + column) as usize], 1);
            pieces &= pieces - 1;
        }
    }

    eval
}

fn side_mobility(pieces: &[Bitboard; 6], occ: Bitboard, mask: Bitboard) -> Eval {
    const MOVE_FNS: [MoveFn; 4] = [knight_moves, bishop_moves, rook_moves, queen_moves];

    let mut eval = Eval(0, 0);
    for i in 0..4 {
        let mut pieces = pieces[i + 1];

        while pieces != 0 {
            let piece = pieces & pieces.wrapping_neg();
            let movement = MOVE_FNS[i](piece, occ) & mask;
            eval.accum(MOBILITY_EVAL[i], popcnt(movement));

            pieces &= pieces - 1;
        }
    }

    eval
}

fn side_doubled_pawn(pawns: Bitboard) -> Eval {
    let mut eval = Eval(0, 0);
    let mut file = consts::A_FILE;
    for doubled in DOUBLED_PAWN_EVAL {
        eval.accum(doubled, popcnt(pawns & file).max(1) - 1);
        file <<= 1;
    }

    eval
}

// Passed pawns from white's perspective
fn white_passed_pawn(side: Bitboard, enemy: Bitboard) -> Eval {
    let mut mask = enemy;
    mask |= mask >> 8;
    mask |= mask >> 16;
    mask |= mask >> 32;

    mask |= ((mask >> 7) & !consts::A_FILE) | ((mask & !consts::A_FILE) >> 9);

    let mut eval = Eval(0, 0);
    let pawns = side & !mask;
    let mut file = consts::A_FILE;
    for _ in 0..8 {
        let index = (pawns & file).leading_zeros();
        if index != 64 {
            eval.accum(PASSED_PAWN_EVAL[(6 - index / 8) as usize], 1);
        }

        file <<= 1;
    }

    eval
}

fn side_open_file(rook: Bitboard, side_pawns: Bitboard, enemy_pawns: Bitboard) -> Eval {
    let mut eval = Eval(0, 0);
    let mut file = consts::A_FILE;
    for _ in 0..8 {
        if (side_pawns | enemy_pawns) & file == 0 {
            eval.accum(OPEN_FILE_EVAL, popcnt(rook & file))
        } else if (side_pawns & file) == 0 {
            eval.accum(SEMI_OPEN_FILE_EVAL, popcnt(rook & file));
        }

        file <<= 1;
    }

    eval
}

pub fn evaluate(board: &Board) -> i32 {
    let mut eval = Eval(0, 0);

    // material
    #[allow(clippy::needless_range_loop)]
    for i in 0..5 {
        let count = popcnt(board.pieces()[0][i]) - popcnt(board.pieces()[1][i]);
        eval.accum(MATERIAL_EVAL[i], count);
    }

    // bishop pair
    if board.pieces()[0][2] & consts::DARK_SQUARES != 0
        && board.pieces()[0][2] & consts::LIGHT_SQUARES != 0
    {
        eval.accum(BISHOP_PAIR_EVAL, 1);
    }

    if board.pieces()[1][2] & consts::DARK_SQUARES != 0
        && board.pieces()[1][2] & consts::LIGHT_SQUARES != 0
    {
        eval.accum(BISHOP_PAIR_EVAL, -1);
    }

    // psts
    eval.accum(side_pst(&board.pieces()[0], 0), 1);
    eval.accum(side_pst(&board.pieces()[1], 3), -1);

    // mobility
    let occ = board.white() | board.black();
    eval.accum(side_mobility(&board.pieces()[0], occ, consts::ALL), 1);
    eval.accum(side_mobility(&board.pieces()[1], occ, consts::ALL), -1);

    // doubled pawns
    eval.accum(side_doubled_pawn(board.pieces()[0][0]), 1);
    eval.accum(side_doubled_pawn(board.pieces()[1][0]), -1);

    // passed pawns
    eval.accum(
        white_passed_pawn(board.pieces()[0][0], board.pieces()[1][0]),
        1,
    );

    eval.accum(
        white_passed_pawn(
            board.pieces()[1][0].swap_bytes(),
            board.pieces()[0][0].swap_bytes(),
        ),
        -1,
    );

    // open files
    eval.accum(
        side_open_file(
            board.pieces()[0][3],
            board.pieces()[0][0],
            board.pieces()[1][0],
        ),
        1,
    );

    eval.accum(
        side_open_file(
            board.pieces()[1][3],
            board.pieces()[1][0],
            board.pieces()[0][0],
        ),
        -1,
    );

    resolve(board, eval)
}
