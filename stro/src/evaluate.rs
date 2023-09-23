use crate::consts;
use crate::movegen::{bishop_moves, knight_moves, queen_moves, rook_moves, MoveFn};
use crate::position::{Bitboard, Board, Color};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
struct Eval(i16, i16);

pub const MAX_EVAL: i32 = 128 * 256 - 1;
pub const MIN_EVAL: i32 = -MAX_EVAL;

#[rustfmt::skip]
const MATERIAL_EVAL: [Eval; 5] = [
    Eval( 312,  318),
    Eval( 748,  679),
    Eval( 804,  699),
    Eval(1136, 1244),
    Eval(2506, 2361),
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(  22,   15),
    Eval(  22,   10),
    Eval(  16,    7),
    Eval(  14,   -1),
];

const BISHOP_PAIR_EVAL: Eval = Eval(86, 178);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval(   1,   23),
        Eval( -11,   -4),
        Eval(  24,  -10),
        Eval(  26,  -13),
        Eval(  -8,   -9),
        Eval(  -4,   16),
        Eval(   0,    0),
    ],
    [
        Eval( -29,  -50),
        Eval(  -3,  -26),
        Eval(  -2,    2),
        Eval(  37,   39),
        Eval(  39,   40),
        Eval(  -1,    3),
        Eval(  -1,  -22),
        Eval( -23,  -53),
    ],
    [
        Eval(   1,  -22),
        Eval(   7,  -15),
        Eval(  17,    7),
        Eval(  -4,   22),
        Eval(   1,   21),
        Eval(  10,   13),
        Eval(  -3,  -15),
        Eval( -14,  -30),
    ],
    [
        Eval(   4,  -14),
        Eval( -35,   31),
        Eval(   2,    8),
        Eval(  14,   17),
        Eval(  16,   17),
        Eval(  -1,   17),
        Eval( -38,   37),
        Eval(  -6,  -15),
    ],
    [
        Eval(  15,  -58),
        Eval(   5,  -29),
        Eval( -24,   37),
        Eval( -26,   55),
        Eval( -30,   57),
        Eval( -19,   33),
        Eval(   0,  -26),
        Eval(  13,  -62),
    ],
    [
        Eval(  14,  -45),
        Eval( -26,    8),
        Eval( -45,   35),
        Eval(  37,   44),
        Eval(  32,   45),
        Eval( -63,   37),
        Eval( -51,    7),
        Eval(  -2,  -48),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -33,  -10),
        Eval(  29,   22),
        Eval(   1,   12),
        Eval(  20,   -2),
        Eval(  13,   -1),
        Eval(  -9,   15),
        Eval(  19,   22),
        Eval( -42,   -9),
    ],
    [
        Eval( -30,  -20),
        Eval(  15,   -3),
        Eval(  -1,  -15),
        Eval(   3,    2),
        Eval(   8,   -1),
        Eval( -10,  -11),
        Eval(   9,   -4),
        Eval( -30,  -17),
    ],
    [
        Eval(  31,  -19),
        Eval(  17,   -5),
        Eval(  -3,   -8),
        Eval( -11,    1),
        Eval( -17,   -2),
        Eval( -17,   -7),
        Eval(  29,  -11),
        Eval(  23,  -19),
    ],
    [
        Eval( -28,    3),
        Eval( -20,   14),
        Eval(  12,    2),
        Eval(  38,  -13),
        Eval(  36,  -12),
        Eval(  13,    0),
        Eval( -18,    7),
        Eval( -28,    3),
    ],
    [
        Eval(  33,  -35),
        Eval(   2,    2),
        Eval(  -3,   15),
        Eval(   1,   12),
        Eval(  -3,   18),
        Eval(  -5,   10),
        Eval(   7,   -1),
        Eval(  31,  -33),
    ],
    [
        Eval(  -7,  -26),
        Eval(  39,    2),
        Eval( -59,   21),
        Eval( -40,    7),
        Eval( -46,   12),
        Eval( -60,   20),
        Eval(  43,   -2),
        Eval(   2,  -33),
    ],
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval(-114, -112),
    Eval( -87,  -85),
    Eval( -77,  -66),
    Eval( -79,  -54),
    Eval( -58,  -49),
    Eval( -53,  -76),
    Eval( -58,  -90),
    Eval( -83, -104),
];

// The tuner game positive values for some of these, but they
// are clamped at zero beause the asm can't handle them
#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(   0,    0),
    Eval( -28,   -5),
    Eval( -43,  -19),
    Eval( -68,  -24),
    Eval( -88,  -27),
    Eval( -45,  -21),
    Eval( -58,  -24),
    Eval( -51,    0),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval( -39,  -68),
    Eval( -59,  -48),
    Eval( -52,    1),
    Eval(  45,   43),
    Eval( 110,   93),
    Eval( 125,  121),
];

const OPEN_FILE_EVAL: Eval = Eval(70, -4);
const SEMI_OPEN_FILE_EVAL: Eval = Eval(36, -16);

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

            eval.accum(RANK_PST[i][((index as u8 >> 3) ^ row_mask) as usize], 1);
            eval.accum(FILE_PST[i][(index & 0b111) as usize], 1);
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
