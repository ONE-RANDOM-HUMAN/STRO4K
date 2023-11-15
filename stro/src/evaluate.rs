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
    Eval(  88,  121),
    Eval( 282,  294),
    Eval( 300,  292),
    Eval( 406,  538),
    Eval( 952,  927),
];

const BISHOP_PAIR_EVAL: Eval = Eval(23, 61);
const TEMPO: Eval = Eval(19, 3);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval( -11,   -4),
        Eval( -16,  -12),
        Eval(   3,  -15),
        Eval(  15,   -9),
        Eval(  32,    9),
        Eval(  46,   60),
        Eval(   0,    0),
    ],
    [
        Eval( -23,  -20),
        Eval( -11,  -12),
        Eval( -11,   -5),
        Eval(   3,   10),
        Eval(  18,   14),
        Eval(  52,    3),
        Eval(  41,   -1),
        Eval( -46,    4),
    ],
    [
        Eval( -10,   -9),
        Eval(  -1,   -6),
        Eval(   2,   -3),
        Eval(   1,    2),
        Eval(   4,    6),
        Eval(  31,    0),
        Eval(  -2,    4),
        Eval( -35,   13),
    ],
    [
        Eval( -14,  -14),
        Eval( -26,  -10),
        Eval( -21,   -5),
        Eval( -17,    3),
        Eval(   4,    7),
        Eval(  28,    5),
        Eval(  40,   10),
        Eval(  49,    2),
    ],
    [
        Eval(  -5,  -45),
        Eval(   0,  -39),
        Eval(  -8,  -11),
        Eval( -12,   14),
        Eval(  -5,   29),
        Eval(  20,   28),
        Eval(   4,   35),
        Eval(  43,   -4),
    ],
    [
        Eval(   2,  -26),
        Eval(  -8,   -6),
        Eval( -26,    3),
        Eval( -12,    7),
        Eval(   8,   14),
        Eval(  32,   22),
        Eval(  48,   17),
        Eval(  50,   -4),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -18,    1),
        Eval(  -5,   11),
        Eval( -11,    4),
        Eval(   2,   -6),
        Eval(   3,    1),
        Eval(  18,   -5),
        Eval(  13,    1),
        Eval(  -5,  -14),
    ],
    [
        Eval( -13,  -14),
        Eval(  -2,   -6),
        Eval(  -5,    3),
        Eval(   3,    7),
        Eval(   2,    7),
        Eval(   4,   -2),
        Eval(   6,    1),
        Eval(  -1,   -8),
    ],
    [
        Eval(   1,   -1),
        Eval(   3,    1),
        Eval(  -3,    0),
        Eval(  -5,    4),
        Eval(  -3,    1),
        Eval(  -6,    0),
        Eval(  10,   -2),
        Eval(  10,   -6),
    ],
    [
        Eval( -13,    3),
        Eval(  -9,    3),
        Eval(  -1,    4),
        Eval(   7,    2),
        Eval(   9,   -3),
        Eval(   0,    1),
        Eval(   6,   -5),
        Eval(   2,   -9),
    ],
    [
        Eval(  -6,  -28),
        Eval(  -5,  -10),
        Eval(  -5,    2),
        Eval(  -5,   13),
        Eval(  -5,   15),
        Eval(   0,   14),
        Eval(  16,    0),
        Eval(  23,   -8),
    ],
    [
        Eval(  19,  -16),
        Eval(  23,   -2),
        Eval(  -1,    4),
        Eval( -33,   12),
        Eval(  -1,    2),
        Eval( -42,   11),
        Eval(  14,   -2),
        Eval(  11,  -14),
    ],
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(   6,    3),
    Eval(   5,    4),
    Eval(   4,    1),
    Eval(   2,    1),
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval( -43,  -47),
    Eval( -21,  -33),
    Eval( -21,  -21),
    Eval( -23,  -13),
    Eval( -15,  -19),
    Eval( -26,  -26),
    Eval( -17,  -34),
    Eval( -28,  -44),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(   1,    1),
    Eval( -10,   -7),
    Eval( -10,  -10),
    Eval( -20,  -13),
    Eval( -17,  -16),
    Eval( -16,   -7),
    Eval( -11,   -8),
    Eval( -12,   -2),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval(  -3,   -4),
    Eval(  -9,    4),
    Eval(  -6,   20),
    Eval(  16,   31),
    Eval(  39,   42),
    Eval(  46,   60),
];

#[rustfmt::skip]
const OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -1,  -10),
    Eval(  -5,   -3),
    Eval(  25,   -1),
    Eval(  -7,   10),
    Eval( -37,   -9),
];

#[rustfmt::skip]
const SEMI_OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -2,    7),
    Eval(  -4,   19),
    Eval(  13,   10),
    Eval(   3,    7),
    Eval( -12,   14),
];

#[rustfmt::skip]
const PAWN_SHIELD_EVAL: [Eval; 5] = [
    Eval( -38,   15),
    Eval(  -7,   -1),
    Eval(  18,   -5),
    Eval(  36,   -3),
    Eval(  29,    5),
];

#[rustfmt::skip]
const PAWN_DEFENDED_EVAL: [Eval; 6] = [
    Eval(  11,    4),
    Eval(   1,   11),
    Eval(   1,   13),
    Eval(   1,   20),
    Eval(  -5,   21),
    Eval( -33,   26),
];

#[rustfmt::skip]
const PAWN_ATTACKED_EVAL: [Eval; 6] = [
    Eval(   5,   20),
    Eval( -40,  -29),
    Eval( -40,  -39),
    Eval( -39,  -34),
    Eval( -30,  -18),
    Eval( -34,   -7),
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
