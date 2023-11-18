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
    Eval( 112,  212),
    Eval( 322,  481),
    Eval( 353,  493),
    Eval( 468,  886),
    Eval(1292, 1293),
];

const BISHOP_PAIR_EVAL: Eval = Eval(31, 95);
const TEMPO: Eval = Eval(29, 10);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval( -18,  -22),
        Eval( -24,  -35),
        Eval(   3,  -37),
        Eval(  23,  -26),
        Eval(  40,   12),
        Eval(  59,   88),
        Eval(   0,    0),
    ],
    [
        Eval( -30,   -8),
        Eval( -14,   -7),
        Eval( -14,   -4),
        Eval(   6,   18),
        Eval(  24,   24),
        Eval(  72,    6),
        Eval(  49,    7),
        Eval( -64,   15),
    ],
    [
        Eval( -14,   -1),
        Eval(  -2,   -2),
        Eval(   4,    0),
        Eval(   4,    3),
        Eval(   6,   11),
        Eval(  41,    5),
        Eval(  -3,   13),
        Eval( -47,   27),
    ],
    [
        Eval( -17,   -7),
        Eval( -29,   -3),
        Eval( -22,    2),
        Eval( -17,   18),
        Eval(   8,   25),
        Eval(  35,   23),
        Eval(  46,   34),
        Eval(  60,   15),
    ],
    [
        Eval(  -1,  -16),
        Eval(  11,  -23),
        Eval(   1,    2),
        Eval(  -6,   30),
        Eval(   0,   48),
        Eval(  28,   59),
        Eval(  13,   71),
        Eval(  50,   26),
    ],
    [
        Eval(  11,  -58),
        Eval(  -3,  -21),
        Eval( -35,    0),
        Eval( -33,   21),
        Eval( -12,   42),
        Eval(  37,   60),
        Eval(  58,   50),
        Eval(  69,    7),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -25,    2),
        Eval(  -3,   21),
        Eval( -16,    3),
        Eval(   4,  -10),
        Eval(   7,   -3),
        Eval(  21,   -4),
        Eval(  13,    9),
        Eval(  -8,  -21),
    ],
    [
        Eval( -16,   -1),
        Eval(   0,    3),
        Eval(  -6,    8),
        Eval(   6,   13),
        Eval(   3,   12),
        Eval(   5,   -2),
        Eval(   9,   11),
        Eval(  -1,    3),
    ],
    [
        Eval(   5,    5),
        Eval(   8,    4),
        Eval(  -5,    4),
        Eval(  -7,   10),
        Eval(  -6,    6),
        Eval(  -8,    5),
        Eval(  15,    3),
        Eval(  14,    1),
    ],
    [
        Eval( -18,   19),
        Eval( -15,   20),
        Eval(   1,   22),
        Eval(  12,   15),
        Eval(  13,    8),
        Eval(  -3,   14),
        Eval(   3,    8),
        Eval(   5,   -1),
    ],
    [
        Eval(   0,   12),
        Eval(   0,   19),
        Eval(   3,   25),
        Eval(  -1,   37),
        Eval(   0,   37),
        Eval(   6,   42),
        Eval(  23,   33),
        Eval(  36,   38),
    ],
    [
        Eval(  35,  -37),
        Eval(  33,   -4),
        Eval(  -1,   14),
        Eval( -52,   27),
        Eval(  -3,   10),
        Eval( -59,   19),
        Eval(  23,   -8),
        Eval(  19,  -34),
    ],
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(   8,   10),
    Eval(   7,    8),
    Eval(   4,    4),
    Eval(   2,   10),
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval( -49,  -86),
    Eval( -26,  -59),
    Eval( -22,  -37),
    Eval( -30,  -26),
    Eval( -15,  -37),
    Eval( -31,  -49),
    Eval( -17,  -62),
    Eval( -29,  -82),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(   0,   11),
    Eval( -16,  -10),
    Eval( -14,  -15),
    Eval( -27,  -23),
    Eval( -24,  -21),
    Eval( -24,   -8),
    Eval( -16,  -10),
    Eval( -25,   10),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval(  -3,  -15),
    Eval( -11,    0),
    Eval( -10,   30),
    Eval(  16,   51),
    Eval(  45,   67),
    Eval(  59,   88),
];

#[rustfmt::skip]
const OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -3,   -9),
    Eval(  -7,    2),
    Eval(  32,    4),
    Eval( -12,   10),
    Eval( -50,  -13),
];

#[rustfmt::skip]
const SEMI_OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -4,   13),
    Eval(  -8,   26),
    Eval(  16,   17),
    Eval(   2,   16),
    Eval( -16,   16),
];

#[rustfmt::skip]
const PAWN_SHIELD_EVAL: [Eval; 5] = [
    Eval( -41,    9),
    Eval(  -9,    1),
    Eval(  22,   -4),
    Eval(  49,   -3),
    Eval(  41,    6),
];

#[rustfmt::skip]
const PAWN_DEFENDED_EVAL: [Eval; 6] = [
    Eval(  14,   11),
    Eval(   2,   12),
    Eval(   2,   18),
    Eval(   4,   19),
    Eval(  -6,   30),
    Eval( -45,   33),
];

#[rustfmt::skip]
const PAWN_ATTACKED_EVAL: [Eval; 6] = [
    Eval(   7,   27),
    Eval( -53,  -45),
    Eval( -50,  -59),
    Eval( -48,  -48),
    Eval( -41,  -10),
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
