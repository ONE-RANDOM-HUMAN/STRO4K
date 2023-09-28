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
    Eval( 330,  339),
    Eval( 762,  691),
    Eval( 830,  715),
    Eval(1219, 1290),
    Eval(2562, 2362),
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(  28,   16),
    Eval(  23,    9),
    Eval(  14,    4),
    Eval(  12,   -1),
];

const BISHOP_PAIR_EVAL: Eval = Eval(90, 176);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval( -44,  -18),
        Eval( -41,  -46),
        Eval(   6,  -57),
        Eval(  43,  -34),
        Eval(  73,   32),
        Eval(  98,   91),
        Eval(   0,    0),
    ],
    [
        Eval( -46,  -46),
        Eval( -26,  -38),
        Eval( -34,  -19),
        Eval(   6,   31),
        Eval(  45,   45),
        Eval(  95,   27),
        Eval( 100,   21),
        Eval( -74,   25),
    ],
    [
        Eval( -17,  -32),
        Eval(   2,  -25),
        Eval(  13,   -3),
        Eval(  -1,   12),
        Eval(   1,   30),
        Eval(  69,   10),
        Eval(  -5,   10),
        Eval( -65,   30),
    ],
    [
        Eval( -28,  -37),
        Eval( -68,  -25),
        Eval( -54,   -7),
        Eval( -39,   19),
        Eval(  30,   32),
        Eval(  72,   28),
        Eval(  79,   49),
        Eval(  97,   25),
    ],
    [
        Eval(   2,  -90),
        Eval(   9,  -89),
        Eval( -24,   -5),
        Eval( -36,   60),
        Eval(  -9,   83),
        Eval(  58,   73),
        Eval(  27,   85),
        Eval(  92,   13),
    ],
    [
        Eval(  -4,  -64),
        Eval( -41,  -12),
        Eval( -79,    9),
        Eval( -14,   26),
        Eval(  55,   44),
        Eval(  82,   64),
        Eval(  92,   58),
        Eval( 100,   20),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -56,   24),
        Eval(  -6,   42),
        Eval( -40,   28),
        Eval(  13,    0),
        Eval(  10,   14),
        Eval(  45,   -3),
        Eval(  45,    3),
        Eval( -20,  -29),
    ],
    [
        Eval( -22,  -20),
        Eval(  -1,   -8),
        Eval( -11,   -4),
        Eval(  11,    9),
        Eval(   5,    5),
        Eval(   7,  -17),
        Eval(  18,    8),
        Eval(   8,   -9),
    ],
    [
        Eval(  12,  -12),
        Eval(  18,   -3),
        Eval(  -8,   -6),
        Eval( -11,    4),
        Eval( -18,    3),
        Eval( -21,    1),
        Eval(  35,  -15),
        Eval(  34,  -29),
    ],
    [
        Eval( -33,    9),
        Eval( -26,   16),
        Eval(  12,   12),
        Eval(  33,    6),
        Eval(  31,   -6),
        Eval(   3,    5),
        Eval(  -1,    7),
        Eval( -13,  -12),
    ],
    [
        Eval(  -8,  -64),
        Eval( -16,  -20),
        Eval(  -7,    6),
        Eval(  -4,   24),
        Eval( -13,   40),
        Eval(  -5,   34),
        Eval(  43,    6),
        Eval(  65,  -10),
    ],
    [
        Eval(  47,  -34),
        Eval(  73,    4),
        Eval(  10,   18),
        Eval( -81,   30),
        Eval( -13,    5),
        Eval( -93,   27),
        Eval(  47,  -10),
        Eval(  21,  -38),
    ],
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval(-105, -108),
    Eval( -78,  -85),
    Eval( -65,  -63),
    Eval( -81,  -48),
    Eval( -64,  -47),
    Eval( -89,  -69),
    Eval( -73,  -78),
    Eval( -86,  -91),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(  -8,    0),
    Eval( -38,  -16),
    Eval( -42,  -32),
    Eval( -79,  -31),
    Eval( -65,  -50),
    Eval( -47,  -26),
    Eval( -35,  -30),
    Eval( -45,   -7),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval( -24,  -32),
    Eval( -44,   -8),
    Eval( -28,   36),
    Eval(  46,   53),
    Eval(  86,   56),
    Eval(  98,   91),
];

#[rustfmt::skip]
const OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -8,  -23),
    Eval( -19,    0),
    Eval(  69,   -6),
    Eval( -22,   28),
    Eval( -87,  -17),
];

#[rustfmt::skip]
const SEMI_OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -5,   11),
    Eval( -14,   40),
    Eval(  43,    1),
    Eval(  10,    7),
    Eval( -38,   29),
];

#[rustfmt::skip]
const PAWN_SHIELD_EVAL: [Eval; 3] = [
    Eval( -31,  -16),
    Eval(  25,  -21),
    Eval(  74,   -9),
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
        let pawn_count = (pawns & (KS_AREA << 8)).count_ones().min(3);

        if pawn_count != 0 {
            eval.accum(PAWN_SHIELD_EVAL[pawn_count as usize - 1], 1);
        }
    } else if king & QS_AREA != 0 {
        let pawn_count = (pawns & (QS_AREA << 8)).count_ones().min(3);

        if pawn_count != 0 {
            eval.accum(PAWN_SHIELD_EVAL[pawn_count as usize - 1], 1);
        }
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
