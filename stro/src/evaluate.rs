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
    Eval( 333,  341),
    Eval( 828,  699),
    Eval( 892,  701),
    Eval(1292, 1305),
    Eval(2929, 2226),
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(  26,   12),
    Eval(  22,   10),
    Eval(  14,    4),
    Eval(  10,    2),
];

const BISHOP_PAIR_EVAL: Eval = Eval(88, 179);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval( -50,  -11),
        Eval( -47,  -36),
        Eval(   5,  -49),
        Eval(  44,  -29),
        Eval(  84,   26),
        Eval( 105,  103),
        Eval(   0,    0),
    ],
    [
        Eval( -57,  -60),
        Eval( -30,  -45),
        Eval( -37,  -20),
        Eval(   5,   30),
        Eval(  48,   42),
        Eval( 102,   24),
        Eval(  90,    8),
        Eval( -73,   10),
    ],
    [
        Eval( -22,  -32),
        Eval(  -2,  -22),
        Eval(  10,   -3),
        Eval(  -2,   12),
        Eval(  -4,   26),
        Eval(  71,    7),
        Eval(  -8,    7),
        Eval( -62,   23),
    ],
    [
        Eval( -30,  -49),
        Eval( -72,  -31),
        Eval( -53,  -17),
        Eval( -38,   10),
        Eval(  25,   21),
        Eval(  77,   20),
        Eval(  89,   39),
        Eval(  91,   15),
    ],
    [
        Eval(  -7,  -97),
        Eval(   7,  -92),
        Eval( -26,   -9),
        Eval( -37,   52),
        Eval( -14,   77),
        Eval(  53,   72),
        Eval(  27,   83),
        Eval(  82,   18),
    ],
    [
        Eval(   7,  -71),
        Eval( -27,  -18),
        Eval( -76,    4),
        Eval( -12,   25),
        Eval(  52,   43),
        Eval(  89,   63),
        Eval(  95,   57),
        Eval(  91,   20),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -51,   11),
        Eval(  -8,   32),
        Eval( -38,   17),
        Eval(  15,  -12),
        Eval(  14,    3),
        Eval(  46,  -11),
        Eval(  42,  -10),
        Eval( -20,  -35),
    ],
    [
        Eval( -29,  -24),
        Eval(  -2,  -10),
        Eval( -12,    5),
        Eval(   9,   14),
        Eval(   4,   11),
        Eval(   7,  -12),
        Eval(  18,    9),
        Eval(   1,  -13),
    ],
    [
        Eval(   7,   -6),
        Eval(  14,    0),
        Eval( -10,    0),
        Eval( -18,    9),
        Eval( -19,    7),
        Eval( -22,    3),
        Eval(  37,   -8),
        Eval(  35,  -20),
    ],
    [
        Eval( -36,    6),
        Eval( -30,    8),
        Eval(  10,    9),
        Eval(  33,    1),
        Eval(  30,  -11),
        Eval(   2,    1),
        Eval(  -2,   -5),
        Eval(  -9,  -19),
    ],
    [
        Eval( -15,  -64),
        Eval( -20,  -22),
        Eval(  -9,   -1),
        Eval(  -8,   19),
        Eval( -15,   32),
        Eval(  -1,   28),
        Eval(  42,    4),
        Eval(  62,  -16),
    ],
    [
        Eval(  43,  -31),
        Eval(  71,    1),
        Eval(  15,   16),
        Eval( -84,   28),
        Eval(  -9,    2),
        Eval( -95,   24),
        Eval(  53,   -9),
        Eval(  27,  -38),
    ],
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval(-105, -100),
    Eval( -77,  -81),
    Eval( -69,  -55),
    Eval( -81,  -42),
    Eval( -64,  -45),
    Eval( -86,  -68),
    Eval( -60,  -76),
    Eval( -92,  -93),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval( -12,    0), // Tuner gave (-12, 2)
    Eval( -35,  -17),
    Eval( -42,  -29),
    Eval( -84,  -31),
    Eval( -68,  -46),
    Eval( -50,  -28),
    Eval( -34,  -27),
    Eval( -56,   -8),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval( -24,  -29),
    Eval( -42,   -7),
    Eval( -20,   39),
    Eval(  50,   56),
    Eval(  96,   71),
    Eval( 105,  103),
];

#[rustfmt::skip]
const OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -8,  -23),
    Eval( -18,    1),
    Eval(  75,   -7),
    Eval( -22,   29),
    Eval( -94,  -14),
];

#[rustfmt::skip]
const SEMI_OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -4,   11),
    Eval( -16,   39),
    Eval(  45,    1),
    Eval(   6,    9),
    Eval( -39,   28),
];

#[rustfmt::skip]
const PAWN_SHIELD_EVAL: [Eval; 5] = [
    Eval( -91,   29),
    Eval( -38,  -14),
    Eval(  25,  -23),
    Eval(  84,  -12),
    Eval(  45,   16),
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
