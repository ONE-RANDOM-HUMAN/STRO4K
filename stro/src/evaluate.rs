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
    Eval( 339,  331),
    Eval( 842,  712),
    Eval( 908,  705),
    Eval(1320, 1312),
    Eval(2969, 2270),
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(  26,   13),
    Eval(  23,   10),
    Eval(  12,    4),
    Eval(  11,    0),
];

const BISHOP_PAIR_EVAL: Eval = Eval(89, 180);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval( -51,   -3),
        Eval( -46,  -30),
        Eval(   9,  -41),
        Eval(  51,  -24),
        Eval(  95,   27),
        Eval( 127,  127), // Tuner gave (135, 126)
        Eval(   0,    0),
    ],
    [
        Eval( -60,  -59),
        Eval( -30,  -46),
        Eval( -35,  -15),
        Eval(   6,   33),
        Eval(  54,   46),
        Eval( 127,   12), // Tuner gave (132, 12)
        Eval( 112,    3),
        Eval( -99,   13),
    ],
    [
        Eval( -22,  -31),
        Eval(   0,  -25),
        Eval(  11,   -5),
        Eval(  -2,   11),
        Eval(  -1,   29),
        Eval(  79,    4),
        Eval( -10,    8),
        Eval( -80,   27),
    ],
    [
        Eval( -34,  -47),
        Eval( -77,  -30),
        Eval( -58,  -13),
        Eval( -38,   12),
        Eval(  31,   24),
        Eval(  89,   19),
        Eval( 109,   33),
        Eval( 119,   10),
    ],
    [
        Eval(  -3, -123),
        Eval(  10, -113),
        Eval( -29,  -12),
        Eval( -39,   55),
        Eval( -17,   90),
        Eval(  53,   79),
        Eval(  19,   95),
        Eval( 106,    6),
    ],
    [
        Eval(   5,  -78),
        Eval( -29,  -22),
        Eval( -84,    8),
        Eval( -23,   28),
        Eval(  55,   47),
        Eval( 112,   65),
        Eval( 127,   58), // Tuner gave (131, 58)
        Eval( 126,   13),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -54,    9),
        Eval(  -7,   34),
        Eval( -40,   19),
        Eval(  16,  -15),
        Eval(  14,    3),
        Eval(  50,  -11),
        Eval(  42,   -9),
        Eval( -20,  -39),
    ],
    [
        Eval( -29,  -25),
        Eval(  -2,  -11),
        Eval( -14,    4),
        Eval(   9,   17),
        Eval(   5,   16),
        Eval(   9,  -10),
        Eval(  19,    7),
        Eval(   2,  -15),
    ],
    [
        Eval(   6,   -6),
        Eval(  16,    0),
        Eval( -11,    0),
        Eval( -17,   10),
        Eval( -18,    7),
        Eval( -21,    3),
        Eval(  38,  -10),
        Eval(  36,  -22),
    ],
    [
        Eval( -37,    6),
        Eval( -33,   10),
        Eval(   7,    9),
        Eval(  30,    2),
        Eval(  28,  -13),
        Eval(   4,    1),
        Eval(   3,   -4),
        Eval(  -3,  -21),
    ],
    [
        Eval( -13,  -78),
        Eval( -17,  -25),
        Eval( -10,    3),
        Eval( -11,   22),
        Eval( -17,   38),
        Eval(  -1,   32),
        Eval(  46,   -3),
        Eval(  71,  -27),
    ],
    [
        Eval(  48,  -38),
        Eval(  76,   -2),
        Eval(   7,   15),
        Eval( -97,   33),
        Eval(  -6,    5),
        Eval(-117,   31),
        Eval(  49,  -13),
        Eval(  26,  -42),
    ],
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval(-127, -122), // Tuner game (-139, -122)
    Eval( -88,  -88),
    Eval( -74,  -56),
    Eval( -92,  -37),
    Eval( -71,  -44),
    Eval(-104,  -64),
    Eval( -67,  -82),
    Eval(-107, -104),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(  -6,    0), // Tuner game (-6, 2)
    Eval( -39,  -21),
    Eval( -44,  -33),
    Eval( -91,  -32),
    Eval( -71,  -53),
    Eval( -52,  -29),
    Eval( -36,  -32),
    Eval( -53,   -9),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval( -26,  -24),
    Eval( -44,    2),
    Eval( -20,   46),
    Eval(  52,   68),
    Eval( 120,   84),
    Eval( 127,  127), // Tuner gave (135, 126)
];

#[rustfmt::skip]
const OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -6,  -24),
    Eval( -17,    1),
    Eval(  84,  -10),
    Eval( -23,   36),
    Eval(-109,  -11),
];

#[rustfmt::skip]
const SEMI_OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -5,   11),
    Eval( -14,   43),
    Eval(  51,    5),
    Eval(   9,   17),
    Eval( -36,   32),
];

#[rustfmt::skip]
const PAWN_SHIELD_EVAL: [Eval; 5] = [
    Eval(-110,   42),
    Eval( -35,  -11),
    Eval(  36,  -24),
    Eval(  99,  -17),
    Eval(  65,   16),
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
