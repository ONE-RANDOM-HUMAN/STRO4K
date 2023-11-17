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
    Eval(  75,  142),
    Eval( 214,  321),
    Eval( 235,  329),
    Eval( 312,  591),
    Eval( 861,  862),
];

const BISHOP_PAIR_EVAL: Eval = Eval(20, 64);
const TEMPO: Eval = Eval(19, 7);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval( -12,  -14),
        Eval( -16,  -23),
        Eval(   2,  -25),
        Eval(  15,  -18),
        Eval(  27,    8),
        Eval(  39,   58),
        Eval(   0,    0),
    ],
    [
        Eval( -20,   -5),
        Eval(  -9,   -5),
        Eval(  -9,   -2),
        Eval(   4,   12),
        Eval(  16,   16),
        Eval(  48,    4),
        Eval(  33,    5),
        Eval( -43,   10),
    ],
    [
        Eval(  -9,   -1),
        Eval(  -1,   -1),
        Eval(   2,    0),
        Eval(   3,    2),
        Eval(   4,    7),
        Eval(  27,    3),
        Eval(  -2,    9),
        Eval( -31,   18),
    ],
    [
        Eval( -11,   -5),
        Eval( -20,   -2),
        Eval( -15,    1),
        Eval( -12,   12),
        Eval(   5,   17),
        Eval(  23,   15),
        Eval(  30,   23),
        Eval(  40,   10),
    ],
    [
        Eval(   0,  -11),
        Eval(   7,  -16),
        Eval(   0,    1),
        Eval(  -4,   20),
        Eval(   0,   32),
        Eval(  19,   39),
        Eval(   9,   47),
        Eval(  33,   17),
    ],
    [
        Eval(   7,  -39),
        Eval(  -2,  -14),
        Eval( -23,    0),
        Eval( -22,   14),
        Eval(  -8,   28),
        Eval(  25,   40),
        Eval(  38,   33),
        Eval(  46,    5),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -17,    1),
        Eval(  -2,   14),
        Eval( -10,    2),
        Eval(   3,   -6),
        Eval(   5,   -2),
        Eval(  14,   -3),
        Eval(   8,    6),
        Eval(  -5,  -14),
    ],
    [
        Eval( -10,   -1),
        Eval(   0,    2),
        Eval(  -4,    5),
        Eval(   4,    9),
        Eval(   2,    8),
        Eval(   3,   -1),
        Eval(   6,    7),
        Eval(   0,    2),
    ],
    [
        Eval(   4,    4),
        Eval(   5,    2),
        Eval(  -3,    3),
        Eval(  -5,    7),
        Eval(  -4,    4),
        Eval(  -5,    4),
        Eval(  10,    2),
        Eval(   9,    1),
    ],
    [
        Eval( -12,   13),
        Eval( -10,   13),
        Eval(   0,   15),
        Eval(   8,   10),
        Eval(   9,    6),
        Eval(  -2,   10),
        Eval(   2,    5),
        Eval(   3,   -1),
    ],
    [
        Eval(   0,    8),
        Eval(   0,   13),
        Eval(   2,   17),
        Eval(  -1,   25),
        Eval(   0,   25),
        Eval(   4,   28),
        Eval(  15,   22),
        Eval(  24,   25),
    ],
    [
        Eval(  24,  -25),
        Eval(  22,   -2),
        Eval(  -1,   10),
        Eval( -35,   18),
        Eval(  -2,    6),
        Eval( -39,   13),
        Eval(  15,   -5),
        Eval(  13,  -23),
    ],
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(   5,    7),
    Eval(   4,    6),
    Eval(   3,    3),
    Eval(   1,    7),
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval( -32,  -57),
    Eval( -18,  -39),
    Eval( -15,  -25),
    Eval( -20,  -17),
    Eval( -10,  -25),
    Eval( -21,  -32),
    Eval( -11,  -41),
    Eval( -20,  -54),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(   0,    7),
    Eval( -10,   -7),
    Eval(  -9,  -10),
    Eval( -18,  -15),
    Eval( -16,  -14),
    Eval( -16,   -5),
    Eval( -11,   -6),
    Eval( -17,    7),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval(  -2,  -10),
    Eval(  -8,    0),
    Eval(  -7,   20),
    Eval(  10,   34),
    Eval(  30,   44),
    Eval(  39,   58),
];

#[rustfmt::skip]
const OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -2,   -6),
    Eval(  -5,    1),
    Eval(  21,    2),
    Eval(  -8,    6),
    Eval( -33,   -8),
];

#[rustfmt::skip]
const SEMI_OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -3,    9),
    Eval(  -5,   17),
    Eval(  11,   11),
    Eval(   1,   11),
    Eval( -10,   11),
];

#[rustfmt::skip]
const PAWN_SHIELD_EVAL: [Eval; 5] = [
    Eval( -27,    6),
    Eval(  -6,    1),
    Eval(  15,   -3),
    Eval(  33,   -2),
    Eval(  27,    4),
];

#[rustfmt::skip]
const PAWN_DEFENDED_EVAL: [Eval; 6] = [
    Eval(  10,    8),
    Eval(   1,    8),
    Eval(   1,   12),
    Eval(   3,   13),
    Eval(  -4,   20),
    Eval( -30,   22),
];

#[rustfmt::skip]
const PAWN_ATTACKED_EVAL: [Eval; 6] = [
    Eval(   4,   18),
    Eval( -35,  -30),
    Eval( -33,  -39),
    Eval( -32,  -32),
    Eval( -27,   -7),
    Eval(   0,    0),
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
