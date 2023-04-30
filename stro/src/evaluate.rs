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
    Eval(325, 309),
    Eval(824, 754).accum_to(MOBILITY_EVAL[0], -4),
    Eval(906, 789).accum_to(MOBILITY_EVAL[1], -6),
    Eval(1275, 1342).accum_to(MOBILITY_EVAL[2], -7),
    Eval(2594, 2451).accum_to(MOBILITY_EVAL[3], -13),
];

const MOBILITY_EVAL: [Eval; 4] = [Eval(36, 19), Eval(24, 10), Eval(14, 3), Eval(11, 2)];

const BISHOP_PAIR_EVAL: Eval = Eval(109, 160);

const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(  0,   0),
        Eval(-44,  -9),
        Eval(-49, -34),
        Eval(-10, -43),
        Eval( 35, -21),
        Eval( 69,  35),
        Eval( 72,  91),
        Eval(  0,   0),
    ],
    [
        Eval(-34, -33),
        Eval(-28, -37),
        Eval(-49, -28),
        Eval( -6,  15),
        Eval( 39,  29),
        Eval( 68,  10),
        Eval( 38,  15),
        Eval( -7,   3),
    ],
    [
        Eval(-13, -24),
        Eval(  0, -19),
        Eval(  9,  -3),
        Eval( -3,   9),
        Eval( 11,  15),
        Eval( 49,   6),
        Eval( -0,   0),
        Eval( -6,   4),
    ],
    [
        Eval(-27, -45),
        Eval(-71, -32),
        Eval(-47, -20),
        Eval(-27,   4),
        Eval( 19,  23),
        Eval( 43,  26),
        Eval( 57,  44),
        Eval( 39,  23),
    ],
    [
        Eval(-17, -51),
        Eval( -9, -59),
        Eval(-29, -14),
        Eval(-27,  13),
        Eval( -2,  27),
        Eval( 42,  32),
        Eval( 22,  34),
        Eval( 31,  10),
    ],
    [
        Eval( 11, -80),
        Eval(-32, -28),
        Eval(-36, -11),
        Eval(  2,  15),
        Eval( 17,  39),
        Eval( 19,  52),
        Eval( 15,  36),
        Eval(  8,   9),
    ],
];

const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval(-44,  25),
        Eval(-25,  37),
        Eval(-31,  22),
        Eval(  4,   2),
        Eval( 20,   6),
        Eval( 59,   2),
        Eval( 43, -13),
        Eval(  3, -17),
    ],
    [
        Eval(  2,  -2),
        Eval(  3,  -3),
        Eval(-20,  -8),
        Eval(  1,  -1),
        Eval( -5,  -0),
        Eval( -6, -17),
        Eval( 20,   9),
        Eval( 22,  -2),
    ],
    [
        Eval( 13,  -2),
        Eval( 21,  -1),
        Eval( -6,  -1),
        Eval(-16,   4),
        Eval(-17,   4),
        Eval(-13,  -1),
        Eval( 34,  -4),
        Eval( 30, -10),
    ],
    [
        Eval(-35,  10),
        Eval(-28,  12),
        Eval( 11,  12),
        Eval( 35,   3),
        Eval( 31, -10),
        Eval(  3,  -2),
        Eval( -5,   2),
        Eval(-32,  -3),
    ],
    [
        Eval(-17, -28),
        Eval(-19, -10),
        Eval( -1,  -2),
        Eval( -4,   5),
        Eval(-11,  12),
        Eval( -1,  10),
        Eval( 24,   4),
        Eval( 40,   3),
    ],
    [
        Eval( 14, -17),
        Eval( 56,   6),
        Eval( 18,  11),
        Eval(-62,  13),
        Eval(-45,  12),
        Eval(-79,  20),
        Eval( 62, -11),
        Eval( 24, -34),
    ],
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval(-52, -51),
    Eval(-39, -38),
    Eval(-45, -29),
    Eval(-41, -13),
    Eval(-53, -23),
    Eval(-76, -47),
    Eval(-38, -40),
    Eval(-38, -44),
];

const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(-29, -11),
    Eval(-27, -21),
    Eval(-46, -32),
    Eval(-81, -34),
    Eval(-71, -48),
    Eval(-45, -38),
    Eval(-33, -33),
    Eval(-76, -28),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval(  0,   0),
    Eval(  0,   0),
    Eval(  0,  47),
    Eval( 37,  73),
    Eval( 80,  88),
    Eval( 80, 161),
];

const OPEN_FILE_EVAL: Eval = Eval(70, 0);
const SEMI_OPEN_FILE_EVAL: Eval = Eval(37, 1);

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
            let index = pieces.trailing_zeros();
            eval.accum(FILE_PST[i][(index & 0b111) as usize], 1);
            eval.accum(RANK_PST[i][((index as u8 >> 3) ^ row_mask) as usize], 1);
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
    eval.accum(side_pst(&board.pieces()[1], 0b111), -1);

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
