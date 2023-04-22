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
    Eval(319, 304),
    Eval(793, 723).accum_to(MOBILITY_EVAL[0], -4),
    Eval(891, 747).accum_to(MOBILITY_EVAL[1], -6),
    Eval(1237, 1283).accum_to(MOBILITY_EVAL[2], -7),
    Eval(2519, 2339).accum_to(MOBILITY_EVAL[3], -13),
];

const MOBILITY_EVAL: [Eval; 4] = [Eval(35, 17), Eval(22, 9), Eval(16, 2), Eval(13, 0)];

const BISHOP_PAIR_EVAL: Eval = Eval(106, 160);

const PST: [[Eval; 8]; 6] = [
    [
        Eval(-38, -24),
        Eval(-42,  14),
        Eval(-41, -35),
        Eval(-17, -32),
        Eval( 14,   4),
        Eval( 61, -22),
        Eval( 42,  77),
        Eval( 45,  65),
    ],
    [
        Eval(-33, -28),
        Eval(-39, -45),
        Eval(-24,  -1),
        Eval(-45, -18),
        Eval( 60,  20),
        Eval( 39,  17),
        Eval(  7,  -0),
        Eval( 32,  14),
    ],
    [
        Eval( 26, -24),
        Eval(-34, -22),
        Eval( 10,  -7),
        Eval(-10,   5),
        Eval( 12,  16),
        Eval( 36,   6),
        Eval(-10,  -3),
        Eval( -5,   4),
    ],
    [
        Eval(-65, -45),
        Eval(-12, -50),
        Eval(-39, -17),
        Eval(-31,  -7),
        Eval( 29,  15),
        Eval( 37,  20),
        Eval( 43,  31),
        Eval( 56,  34),
    ],
    [
        Eval(-21, -38),
        Eval( -4, -78),
        Eval(-16,  -0),
        Eval(-33,   6),
        Eval( 26,  10),
        Eval( 12,  45),
        Eval(  6,  -2),
        Eval( 30,  34),
    ],
    [
        Eval( 49, -61),
        Eval(-58, -39),
        Eval(-13, -13),
        Eval(-19,   6),
        Eval( 15,  29),
        Eval( 20,  48),
        Eval(  9,   8),
        Eval( 13,  24),
    ],
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval(-65, -45),
    Eval(-37, -34),
    Eval(-75, -30),
    Eval(-54, -24),
    Eval(-56, -22),
    Eval(-52, -47),
    Eval(-13, -39),
    Eval(-34, -43),
];

const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(-26, -18),
    Eval(-15, -15),
    Eval(-54, -32),
    Eval(-32, -43),
    Eval(-68, -40),
    Eval(-92, -37),
    Eval(-48, -29),
    Eval(-36, -32),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval(  0,   0),
    Eval(  0,   0),
    Eval(  0,  43),
    Eval( 24,  57),
    Eval(100, 117),
    Eval( 88, 162),
];

const OPEN_FILE_EVAL: Eval = Eval(73, 0);
const SEMI_OPEN_FILE_EVAL: Eval = Eval(40, 0);

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
    eval.accum(side_pst(&board.pieces()[1], 3), -1);

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
