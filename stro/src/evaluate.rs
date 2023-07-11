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
    Eval(318, 311),
    Eval(814, 747).accum_to(MOBILITY_EVAL[0], -4),
    Eval(914, 783).accum_to(MOBILITY_EVAL[1], -6),
    Eval(1265, 1344).accum_to(MOBILITY_EVAL[2], -7),
    Eval(2603, 2442).accum_to(MOBILITY_EVAL[3], -13),
];

const MOBILITY_EVAL: [Eval; 4] = [Eval(37, 21), Eval(24, 10), Eval(17, 1), Eval(12, 4)];

const BISHOP_PAIR_EVAL: Eval = Eval(112, 161);

#[rustfmt::skip]
const PST: [[Eval; 16]; 6] = [
    [
        Eval(-53,   2),
        Eval(-74,  19),
        Eval( -0,   6),
        Eval( -7, -40),
        Eval(-55, -14),
        Eval(-19, -31),
        Eval( -3, -26),
        Eval( -9, -56),
        Eval( -1,  34),
        Eval( 43,  -9),
        Eval( 75, -26),
        Eval( 38, -15),
        Eval( 41,  75),
        Eval( 40,  56),
        Eval( 16,  33),
        Eval(  5,  29),
    ],
    [
        Eval(-24, -19),
        Eval(-38, -33),
        Eval(-24, -33),
        Eval(-22,  -7),
        Eval(-30,  -4),
        Eval(-43, -13),
        Eval(-48, -20),
        Eval(  8,   9),
        Eval( 39,  13),
        Eval( 49,   9),
        Eval( 28,  23),
        Eval( 70,  29),
        Eval(  2,   4),
        Eval( 23,  14),
        Eval( 20,  12),
        Eval(  5,   1),
    ],
    [
        Eval( 22, -16),
        Eval(-37, -24),
        Eval(-29, -16),
        Eval( 24, -18),
        Eval(  6,   4),
        Eval(  1,   2),
        Eval(-23,   4),
        Eval( 11,  -6),
        Eval(  4,  11),
        Eval( 29,   6),
        Eval( 27,  15),
        Eval( 22,  16),
        Eval( -7,   3),
        Eval( -0,   7),
        Eval(  2,   3),
        Eval( -6,  -5),
    ],
    [
        Eval(-60, -35),
        Eval( -4, -38),
        Eval(-20, -47),
        Eval(-51, -43),
        Eval(-49, -12),
        Eval(-23,  -3),
        Eval(-22,  -9),
        Eval( -5, -19),
        Eval( 13,  23),
        Eval( 28,  31),
        Eval( 29,  20),
        Eval( 29,  10),
        Eval( 30,  38),
        Eval( 46,  44),
        Eval( 34,  33),
        Eval( 33,  31),
    ],
    [
        Eval(-18, -22),
        Eval( -4, -47),
        Eval(-15, -46),
        Eval(-19, -21),
        Eval(-31,  -7),
        Eval(-34,   3),
        Eval(-15,  -1),
        Eval( 16,  -1),
        Eval(-17,  -5),
        Eval( -3,  25),
        Eval( 31,  37),
        Eval( 60,  24),
        Eval(-17,  -2),
        Eval( 13,  23),
        Eval( 31,  31),
        Eval( 34,  12),
    ],
    [
        Eval( 42, -30),
        Eval(-20, -38),
        Eval(-75, -38),
        Eval( 48, -68),
        Eval(  5,  -6),
        Eval( -4,   6),
        Eval(-16,  -3),
        Eval(-17, -21),
        Eval( 11,  22),
        Eval( 12,  40),
        Eval(  9,  42),
        Eval(  6,  23),
        Eval(  6,   7),
        Eval(  9,  22),
        Eval( 11,  28),
        Eval(  5,  14),
    ],
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval(-59, -53),
    Eval(-34, -41),
    Eval(-61, -30),
    Eval(-38, -16),
    Eval(-61, -25),
    Eval(-51, -47),
    Eval(-19, -36),
    Eval(-41, -48),
];

const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(-33, -20),
    Eval(-22, -22),
    Eval(-58, -29),
    Eval(-64, -41),
    Eval(-87, -45),
    Eval(-41, -39),
    Eval(-27, -21),
    Eval(-88, -31),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval(  0,   0),
    Eval(  0,   0),
    Eval(  0,  41),
    Eval( 29,  58),
    Eval(102, 126),
    Eval(102, 193),
];

const OPEN_FILE_EVAL: Eval = Eval(73, 0);
const SEMI_OPEN_FILE_EVAL: Eval = Eval(38, 1);

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
            let index = ((piece_index / 2) & 0b11) | ((piece_index / 4) & 0b1100);

            eval.accum(PST[i][(index as u8 ^ row_mask) as usize], 1);
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

fn side_pawn_structure(pawns: Bitboard) -> Eval {
    let mut eval = Eval(0, 0);
    let mut file = consts::A_FILE;
    for i in 0..8 {
        let pawn_count = popcnt(pawns & file);
        let adjacent = ((file << 1) & !consts::A_FILE) | ((file & !consts::A_FILE) >> 1);
        if pawns & adjacent == 0 {
            eval.accum(ISOLATED_PAWN_EVAL[i], pawn_count);
        }

        eval.accum(DOUBLED_PAWN_EVAL[i], pawn_count.max(1) - 1);
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
    eval.accum(side_pst(&board.pieces()[1], 0b1100), -1);

    // mobility
    let occ = board.white() | board.black();
    eval.accum(side_mobility(&board.pieces()[0], occ, consts::ALL), 1);
    eval.accum(side_mobility(&board.pieces()[1], occ, consts::ALL), -1);

    // doubled pawns
    eval.accum(side_pawn_structure(board.pieces()[0][0]), 1);
    eval.accum(side_pawn_structure(board.pieces()[1][0]), -1);

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
