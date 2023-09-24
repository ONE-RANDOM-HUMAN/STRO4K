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
    Eval( 339,  333),
    Eval( 759,  683),
    Eval( 817,  714),
    Eval(1202, 1285),
    Eval(2548, 2360),
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(  27,   15),
    Eval(  23,    9),
    Eval(  14,    4),
    Eval(  12,   -1),
];

const BISHOP_PAIR_EVAL: Eval = Eval(90, 175);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval( -39,  -18),
        Eval( -39,  -47),
        Eval(   1,  -59),
        Eval(  39,  -33),
        Eval(  71,   31),
        Eval(  97,   90),
        Eval(   0,    0),
    ],
    [
        Eval( -45,  -48),
        Eval( -26,  -39),
        Eval( -34,  -18),
        Eval(   6,   31),
        Eval(  44,   46),
        Eval(  94,   26),
        Eval(  99,   19),
        Eval( -74,   22),
    ],
    [
        Eval( -16,  -30),
        Eval(   1,  -24),
        Eval(  12,   -1),
        Eval(  -5,   12),
        Eval(   1,   28),
        Eval(  70,    8),
        Eval(  -6,    8),
        Eval( -64,   30),
    ],
    [
        Eval( -27,  -38),
        Eval( -70,  -26),
        Eval( -56,   -7),
        Eval( -39,   19),
        Eval(  33,   31),
        Eval(  73,   27),
        Eval(  79,   49),
        Eval(  97,   25),
    ],
    [
        Eval(   3,  -90),
        Eval(   9,  -89),
        Eval( -25,   -7),
        Eval( -37,   58),
        Eval( -10,   83),
        Eval(  57,   72),
        Eval(  22,   86),
        Eval(  92,   12),
    ],
    [
        Eval(  -1,  -65),
        Eval( -45,  -13),
        Eval( -79,   13),
        Eval( -12,   27),
        Eval(  56,   44),
        Eval(  82,   64),
        Eval(  91,   59),
        Eval( 100,   22),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -61,   27),
        Eval( -16,   43),
        Eval( -45,   28),
        Eval(  -1,    2),
        Eval(  10,    7),
        Eval(  53,   -5),
        Eval(  51,   -4),
        Eval( -10,  -27),
    ],
    [
        Eval( -22,  -22),
        Eval(   1,  -11),
        Eval(  -9,   -8),
        Eval(  10,    3),
        Eval(   2,    4),
        Eval(   7,  -14),
        Eval(  15,   10),
        Eval(   6,   -8),
    ],
    [
        Eval(  15,  -10),
        Eval(  18,   -2),
        Eval( -10,   -6),
        Eval( -13,    2),
        Eval( -20,    3),
        Eval( -19,    0),
        Eval(  34,  -15),
        Eval(  36,  -27),
    ],
    [
        Eval( -31,    9),
        Eval( -22,   16),
        Eval(  16,   12),
        Eval(  38,    5),
        Eval(  38,   -9),
        Eval(   8,    1),
        Eval(   0,    7),
        Eval( -29,   -6),
    ],
    [
        Eval(  -7,  -64),
        Eval( -15,  -20),
        Eval(  -7,    7),
        Eval(  -5,   23),
        Eval( -15,   41),
        Eval(  -9,   33),
        Eval(  39,    3),
        Eval(  65,  -10),
    ],
    [
        Eval(  56,  -31),
        Eval(  85,    1),
        Eval(  25,   10),
        Eval( -96,   26),
        Eval( -44,   11),
        Eval( -89,   28),
        Eval(  59,  -11),
        Eval(  25,  -34),
    ],
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval(-107, -107),
    Eval( -77,  -82),
    Eval( -69,  -58),
    Eval( -81,  -42),
    Eval( -72,  -40),
    Eval( -95,  -63),
    Eval( -73,  -72),
    Eval( -88,  -89),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(  -7,   -1),
    Eval( -36,  -17),
    Eval( -45,  -33),
    Eval( -77,  -34),
    Eval( -70,  -47),
    Eval( -50,  -29),
    Eval( -42,  -28),
    Eval( -55,   -9),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval( -28,  -35),
    Eval( -47,  -10),
    Eval( -30,   37),
    Eval(  44,   50),
    Eval(  85,   54),
    Eval(  97,   90),
];

const OPEN_FILE_EVAL: Eval = Eval(70, -7);
const SEMI_OPEN_FILE_EVAL: Eval = Eval(45, -2);

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

/// Rank and File psts
/// Each piece has two sets of scores, one for its rank, the other
/// for its file. The sum of these scores acts as the pst.
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
