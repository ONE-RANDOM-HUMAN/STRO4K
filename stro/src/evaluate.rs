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
    Eval( 175,  155),
    Eval( 435,  374),
    Eval( 466,  354),
    Eval( 687,  653),
    Eval(1535, 1157),
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(  12,    2),
    Eval(  11,    5),
    Eval(   6,    2),
    Eval(   6,   -2),
];

const BISHOP_PAIR_EVAL: Eval = Eval(45, 91);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval( -25,    4),
        Eval( -22,   -9),
        Eval(   8,  -17),
        Eval(  32,  -11),
        Eval(  55,   11),
        Eval(  86,   72),
        Eval(   0,    0),
    ],
    [
        Eval( -31,  -36),
        Eval( -10,  -25),
        Eval( -13,   -2),
        Eval(   8,   22),
        Eval(  33,   29),
        Eval(  91,    4),
        Eval(  81,   -8),
        Eval( -87,   10),
    ],
    [
        Eval(  -9,  -17),
        Eval(   3,  -14),
        Eval(   8,   -5),
        Eval(   2,    4),
        Eval(   1,   12),
        Eval(  46,   -2),
        Eval(  -3,    4),
        Eval( -64,   19),
    ],
    [
        Eval( -17,  -20),
        Eval( -40,  -12),
        Eval( -32,   -3),
        Eval( -21,   11),
        Eval(  17,   14),
        Eval(  56,    8),
        Eval(  69,   14),
        Eval(  93,   -3),
    ],
    [
        Eval(   3,  -86),
        Eval(  11,  -75),
        Eval( -13,   -8),
        Eval( -21,   34),
        Eval( -14,   55),
        Eval(  29,   42),
        Eval(   4,   56),
        Eval(  93,  -24),
    ],
    [
        Eval(  -3,  -44),
        Eval( -19,  -14),
        Eval( -53,    9),
        Eval( -26,   21),
        Eval(  31,   27),
        Eval(  90,   32),
        Eval( 127,   20), // Tuner gave (139,  20)
        Eval( 127,  -13), // Tuner gave (171, -13)
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -32,    6),
        Eval(   4,   19),
        Eval( -24,   12),
        Eval(   9,   -9),
        Eval(   6,    5),
        Eval(  30,   -8),
        Eval(  26,   -2),
        Eval( -13,  -20),
    ],
    [
        Eval( -17,  -23),
        Eval(   1,   -8),
        Eval(  -5,    8),
        Eval(   8,   13),
        Eval(   6,   14),
        Eval(   9,   -1),
        Eval(  12,    1),
        Eval(   1,  -18),
    ],
    [
        Eval(   5,   -6),
        Eval(  10,   -3),
        Eval(  -4,   -1),
        Eval(  -8,    4),
        Eval(  -8,    2),
        Eval(  -9,    0),
        Eval(  23,   -7),
        Eval(  22,  -13),
    ],
    [
        Eval( -19,    4),
        Eval( -19,    6),
        Eval(   2,    7),
        Eval(  14,    2),
        Eval(  14,   -7),
        Eval(   2,    1),
        Eval(   5,   -3),
        Eval(   5,  -13),
    ],
    [
        Eval(   0,  -52),
        Eval(  -5,  -16),
        Eval(  -5,    4),
        Eval(  -6,   16),
        Eval(  -7,   25),
        Eval(   2,   19),
        Eval(  31,   -8),
        Eval(  47,  -34),
    ],
    [
        Eval(  30,  -23),
        Eval(  35,   -4),
        Eval(  -7,    9),
        Eval( -52,   21),
        Eval(   5,    3),
        Eval( -75,   18),
        Eval(  20,   -7),
        Eval(  12,  -25),
    ],
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval(-102,  -62),
    Eval( -60,  -45),
    Eval( -45,  -26),
    Eval( -58,  -10),
    Eval( -44,  -18),
    Eval( -70,  -26),
    Eval( -46,  -41),
    Eval( -79,  -49),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(   0,   -1), // Tuner gave (7, -1)
    Eval( -24,  -13),
    Eval( -22,  -19),
    Eval( -49,  -17),
    Eval( -33,  -31),
    Eval( -26,  -14),
    Eval( -16,  -20),
    Eval( -19,   -4),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval( -21,   -4),
    Eval( -29,    9),
    Eval( -14,   32),
    Eval(  24,   44),
    Eval(  70,   56),
    Eval(  86,   72),
];

#[rustfmt::skip]
const OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -2,  -14),
    Eval(  -8,    1),
    Eval(  49,   -8),
    Eval( -13,   27),
    Eval( -62,   -4),
];

#[rustfmt::skip]
const SEMI_OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(   0,    6),
    Eval(  -8,   25),
    Eval(  32,    3),
    Eval(   6,   13),
    Eval( -18,   19),
];

#[rustfmt::skip]
const PAWN_SHIELD_EVAL: [Eval; 5] = [
    Eval( -66,   37),
    Eval(  -7,   -1),
    Eval(  36,  -10),
    Eval(  72,  -11),
    Eval(  69,   -2),
];

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

fn side_open_file(pieces: &[Bitboard; 6], side_pawns: Bitboard, enemy_pawns: Bitboard) -> Eval {
    let mut eval = Eval(0, 0);
    let mut file = consts::A_FILE;
    let mut open = 0;
    let mut semi_open = 0;

    for _ in 0..8 {
        if (side_pawns | enemy_pawns) & file == 0 {
            open |= file;
        } else if (side_pawns & file) == 0 {
            semi_open |= file;
        }

        file <<= 1;
    }

    for (i, piece) in pieces[1..].iter().copied().enumerate() {
        eval.accum(OPEN_FILE_EVAL[i], popcnt(piece & open));
        eval.accum(SEMI_OPEN_FILE_EVAL[i], popcnt(piece & semi_open));
    }

    eval
}

fn white_king_safety(king: Bitboard, pawns: Bitboard) -> Eval {
    let mut eval = Eval(0, 0);

    // Pawn Shield:
    // If the king is on 1st or 2nd rank and not in the middle two files,
    // then give a bonus for up to 3 pawns on the 2rd and 3rd ranks on the
    // same side of the board as the king.
    const QS_AREA: Bitboard = 0x0707;
    const KS_AREA: Bitboard = 0xE0E0;

    if king & KS_AREA != 0 {
        let pawn_count = (pawns & (KS_AREA << 8)).count_ones();
        eval.accum(PAWN_SHIELD_EVAL[pawn_count as usize], 1);
    } else if king & QS_AREA != 0 {
        let pawn_count = (pawns & (QS_AREA << 8)).count_ones();
        eval.accum(PAWN_SHIELD_EVAL[pawn_count as usize], 1);
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
            &board.pieces()[0],
            board.pieces()[0][0],
            board.pieces()[1][0],
        ),
        1,
    );

    eval.accum(
        side_open_file(
            &board.pieces()[1],
            board.pieces()[1][0],
            board.pieces()[0][0],
        ),
        -1,
    );

    eval.accum(
        white_king_safety(board.pieces()[0][5], board.pieces()[0][0]),
        1,
    );

    eval.accum(
        white_king_safety(
            board.pieces()[1][5].swap_bytes(),
            board.pieces()[1][0].swap_bytes(),
        ),
        -1,
    );

    resolve(board, eval)
}
