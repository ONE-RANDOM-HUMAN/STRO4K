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
    Eval( 146,  216),
    Eval( 324,  462),
    Eval( 332,  461),
    Eval( 455,  867),
    Eval(1119, 1475),
];

const BISHOP_PAIR_EVAL: Eval = Eval(60, 107);
const TEMPO: Eval = Eval(80, 41);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval( -22,   -1),
        Eval( -33,  -22),
        Eval(  10,  -29),
        Eval(  43,  -16),
        Eval(  68,   20),
        Eval(  55,  113),
        Eval(   0,    0),
    ],
    [
        Eval( -71,  -43),
        Eval( -31,  -32),
        Eval( -23,   -8),
        Eval(   8,   22),
        Eval(  38,   33),
        Eval( 108,   10),
        Eval(  69,   -5),
        Eval( -57,   -4),
    ],
    [
        Eval( -33,  -16),
        Eval( -11,  -13),
        Eval(   1,   -6),
        Eval(   2,    5),
        Eval(  10,   11),
        Eval(  53,   -1),
        Eval(  -1,    5),
        Eval( -80,   16),
    ],
    [
        Eval( -37,  -33),
        Eval( -62,  -22),
        Eval( -53,  -14),
        Eval( -42,    9),
        Eval(  -1,   26),
        Eval(  36,   16),
        Eval(  47,   24),
        Eval(  33,   10),
    ],
    [
        Eval( -61,  -85),
        Eval( -40,  -90),
        Eval( -49,  -44),
        Eval( -52,   -2),
        Eval( -30,    0),
        Eval(  16,  -33),
        Eval(  -2,  -17),
        Eval(  23,  -71),
    ],
    [
        Eval(  22,  -80),
        Eval(   1,  -35),
        Eval( -35,    1),
        Eval( -39,   33),
        Eval( -50,   68),
        Eval( -25,   91),
        Eval(  66,   45),
        Eval(  79,  -25),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -25,    4),
        Eval(   9,   28),
        Eval( -25,   10),
        Eval(   5,  -12),
        Eval(   9,   -1),
        Eval(  22,   -9),
        Eval(  11,    1),
        Eval( -10,  -24),
    ],
    [
        Eval( -45,  -24),
        Eval(  -7,  -12),
        Eval( -12,    2),
        Eval(   5,    8),
        Eval(   2,   12),
        Eval(   3,   -9),
        Eval(   0,    1),
        Eval( -21,  -19),
    ],
    [
        Eval(   2,  -10),
        Eval(  10,   -5),
        Eval( -14,    2),
        Eval( -17,    9),
        Eval( -16,    3),
        Eval( -19,    0),
        Eval(  15,   -8),
        Eval(  14,  -14),
    ],
    [
        Eval( -40,    0),
        Eval( -41,    7),
        Eval(  -9,    3),
        Eval(   5,   -1),
        Eval(   2,  -12),
        Eval( -24,   -6),
        Eval( -24,   -4),
        Eval( -19,  -16),
    ],
    [
        Eval( -48,  -77),
        Eval( -45,  -58),
        Eval( -34,  -45),
        Eval( -41,  -28),
        Eval( -44,  -27),
        Eval( -32,  -51),
        Eval( -11,  -70),
        Eval(  -5,  -61),
    ],
    [
        Eval(  60,  -57),
        Eval(  54,  -10),
        Eval(   6,   16),
        Eval( -77,   37),
        Eval( -12,   16),
        Eval( -87,   27),
        Eval(  30,  -11),
        Eval(  24,  -42),
    ],
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(   8,    7),
    Eval(  10,    7),
    Eval(   5,    3),
    Eval(   4,   -3),
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval( -86,  -96),
    Eval( -52,  -66),
    Eval( -35,  -41),
    Eval( -62,  -23),
    Eval( -28,  -36),
    Eval( -44,  -51),
    Eval( -19,  -73),
    Eval( -40,  -87),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(   7,    6),
    Eval( -22,  -11),
    Eval( -13,  -17),
    Eval( -36,  -22),
    Eval( -36,  -30),
    Eval( -27,  -12),
    Eval( -22,  -17),
    Eval( -37,    2),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval( -19,  -20),
    Eval( -26,    3),
    Eval( -29,   37),
    Eval(  17,   56),
    Eval(  54,   71),
    Eval(  55,  113),
];

#[rustfmt::skip]
const OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -8,   -6),
    Eval( -17,    7),
    Eval(  46,   -3),
    Eval( -10,   -1),
    Eval( -55,  -21),
];

#[rustfmt::skip]
const SEMI_OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -9,    9),
    Eval( -17,   27),
    Eval(  26,   11),
    Eval(  -1,   23),
    Eval( -21,   20),
];

#[rustfmt::skip]
const PAWN_SHIELD_EVAL: [Eval; 5] = [
    Eval( -55,   32),
    Eval( -15,    1),
    Eval(  29,   -8),
    Eval(  72,   -8),
    Eval(  67,   14),
];

#[rustfmt::skip]
const PAWN_DEFENDED_EVAL: [Eval; 6] = [
    Eval(  24,   13),
    Eval(   3,   14),
    Eval(   0,   23),
    Eval(  14,   20),
    Eval(  -7,   36),
    Eval(-100,   65),
];

#[rustfmt::skip]
const PAWN_ATTACKED_EVAL: [Eval; 6] = [
    Eval(  24,   38),
    Eval( -82,  -89),
    Eval( -74, -104),
    Eval( -66,  -94),
    Eval( -65,  -43),
    Eval( -88,   26),
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

fn pawn_attacked(pieces: &[[Bitboard; 6]; 2]) -> Eval {
    let mut eval = Eval(0, 0);

    let white_pawn_attacks = ((pieces[0][0] << 9) & !consts::A_FILE)
        | ((pieces[0][0] & !consts::A_FILE) << 7);

    let black_pawn_attacks = ((pieces[1][0] >> 7) & !consts::A_FILE)
        | ((pieces[1][0] & !consts::A_FILE) >> 9);

    for (i, piece) in pieces[0].into_iter().enumerate() {
        eval.accum(PAWN_DEFENDED_EVAL[i], popcnt(piece & white_pawn_attacks));
        eval.accum(PAWN_ATTACKED_EVAL[i], popcnt(piece & black_pawn_attacks));
    }

    for (i, piece) in pieces[1].into_iter().enumerate() {
        eval.accum(PAWN_DEFENDED_EVAL[i], -popcnt(piece & black_pawn_attacks));
        eval.accum(PAWN_ATTACKED_EVAL[i], -popcnt(piece & white_pawn_attacks));
    }

    eval
}


pub fn evaluate(board: &Board) -> i32 {
    let mut eval = if board.side_to_move() == Color::White {
        TEMPO
    } else {
        Eval(0, 0).accum_to(TEMPO, -1)
    };

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

    eval.accum(pawn_attacked(board.pieces()), 1);

    resolve(board, eval)
}
