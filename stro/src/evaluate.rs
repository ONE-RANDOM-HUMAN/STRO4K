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
    Eval( 317,  334),
    Eval( 717,  630),
    Eval( 814,  688),
    Eval(1130, 1298),
    Eval(2609, 2298),
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(31, 22),
    Eval(22, 10),
    Eval(19,  1),
    Eval(13,  0),
];

const BISHOP_PAIR_EVAL: Eval = Eval(92, 176);

#[rustfmt::skip]
const PST: [[Eval; 16]; 6] = [
    [
        Eval( -26,   -7),
        Eval( -56,   18),
        Eval(  -7,    6),
        Eval(  -1,  -37),
        Eval( -34,  -18),
        Eval(  -6,  -38),
        Eval(  -4,  -30),
        Eval(  -8,  -49),
        Eval(  12,   24),
        Eval(  36,   -7),
        Eval(  52,  -17),
        Eval(  33,  -17),
        Eval(  54,   52),
        Eval(  54,   51),
        Eval(  48,   48),
        Eval(  21,   42),
    ],
    [
        Eval( -21,  -28),
        Eval( -30,  -33),
        Eval( -19,  -32),
        Eval( -21,  -13),
        Eval( -35,    6),
        Eval( -25,   -8),
        Eval( -29,  -17),
        Eval(   3,   13),
        Eval(  43,   31),
        Eval(  46,   18),
        Eval(  28,   29),
        Eval(  56,   50),
        Eval(   0,   21),
        Eval(  52,   27),
        Eval(  56,   25),
        Eval(  23,    3),
    ],
    [
        Eval(  24,  -20),
        Eval( -25,  -24),
        Eval( -26,  -14),
        Eval(  34,  -23),
        Eval(  11,    7),
        Eval(  10,    7),
        Eval( -15,    5),
        Eval(  11,   -7),
        Eval(   1,   19),
        Eval(  36,    9),
        Eval(  30,   16),
        Eval(  21,   30),
        Eval( -29,   15),
        Eval(  -9,   19),
        Eval(   3,   14),
        Eval(  -4,    4),
    ],
    [
        Eval( -35,  -32),
        Eval(  18,  -37),
        Eval(  10,  -43),
        Eval( -37,  -38),
        Eval( -40,    4),
        Eval( -15,    7),
        Eval( -13,   -2),
        Eval(  12,  -15),
        Eval(  26,   30),
        Eval(  41,   36),
        Eval(  48,   31),
        Eval(  55,   14),
        Eval(  48,   41),
        Eval(  56,   49),
        Eval(  57,   42),
        Eval(  59,   36),
    ],
    [
        Eval( -21,  -51),
        Eval(   1,  -57),
        Eval( -13,  -59),
        Eval( -31,  -56),
        Eval( -37,  -15),
        Eval( -35,   16),
        Eval( -25,    5),
        Eval(  18,   -2),
        Eval( -30,   -1),
        Eval( -20,   53),
        Eval(  38,   55),
        Eval(  53,   50),
        Eval( -33,   -3),
        Eval(  24,   47),
        Eval(  52,   50),
        Eval(  59,   34),
    ],
    [
        Eval(  52,  -28),
        Eval( -14,  -28),
        Eval( -47,  -30),
        Eval(  42,  -44),
        Eval(  18,   -2),
        Eval(  -5,   12),
        Eval( -28,   12),
        Eval( -30,   -8),
        Eval(  51,   36),
        Eval(  47,   40),
        Eval(  47,   41),
        Eval(  41,   30),
        Eval(  54,   22),
        Eval(  54,   41),
        Eval(  55,   47),
        Eval(  51,   32),
    ],
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval(-125, -117),
    Eval( -67,  -78),
    Eval( -96,  -54),
    Eval( -84,  -47),
    Eval( -84,  -44),
    Eval( -64,  -70),
    Eval( -24,  -71),
    Eval( -98, -104),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval( -17,   -9),
    Eval( -13,   -5),
    Eval( -50,  -22),
    Eval( -62,  -36),
    Eval( -88,  -35),
    Eval( -45,  -31),
    Eval( -40,  -13),
    Eval( -76,  -24),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval( -30,  -44),
    Eval( -58,  -15),
    Eval( -23,   25),
    Eval(  40,   32),
    Eval( 110,   89),
    Eval( 126,  122),
];

const OPEN_FILE_EVAL: Eval = Eval(72, -5);
const SEMI_OPEN_FILE_EVAL: Eval = Eval(44, -4);

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
