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
    Eval( 163,  247),
    Eval( 286,  422),
    Eval( 341,  454),
    Eval( 441,  851),
    Eval(1036, 1207),
];

const BISHOP_PAIR_EVAL: Eval = Eval(63, 114);
const TEMPO: Eval = Eval(85, 44);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval( -29,  -16),
        Eval( -40,  -38),
        Eval(   1,  -43),
        Eval(  29,  -25),
        Eval(  43,   25),
        Eval(  57,   62),
        Eval(   0,    0),
    ],
    [
        Eval( -47,  -27),
        Eval( -25,  -24),
        Eval( -25,  -17),
        Eval(   2,   19),
        Eval(  31,   30),
        Eval(  63,   22),
        Eval(  54,   17),
        Eval( -26,   14),
    ],
    [
        Eval( -29,  -13),
        Eval(  -8,   -5),
        Eval(   3,    1),
        Eval(   6,    9),
        Eval(  12,   17),
        Eval(  45,    9),
        Eval(   2,    9),
        Eval( -49,   19),
    ],
    [
        Eval( -23,  -30),
        Eval( -45,  -16),
        Eval( -34,   -9),
        Eval( -20,   16),
        Eval(  14,   29),
        Eval(  40,   27),
        Eval(  45,   38),
        Eval(  35,   19),
    ],
    [
        Eval( -25,  -26),
        Eval(  -3,  -37),
        Eval( -14,   -8),
        Eval( -15,   21),
        Eval(   3,   27),
        Eval(  39,   17),
        Eval(  27,   37),
        Eval(  44,    4),
    ],
    [
        Eval(   4,  -55),
        Eval( -13,  -20),
        Eval( -40,   -4),
        Eval( -22,   26),
        Eval(  13,   48),
        Eval(  46,   58),
        Eval(  50,   40),
        Eval(  43,  -20),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -26,   21),
        Eval(   5,   30),
        Eval( -25,   18),
        Eval(   3,    2),
        Eval(   7,    7),
        Eval(  20,    0),
        Eval(  17,    6),
        Eval( -15,  -14),
    ],
    [
        Eval( -21,   -2),
        Eval(  -2,    1),
        Eval( -13,    2),
        Eval(   4,    9),
        Eval(  -1,    9),
        Eval(  -1,   -9),
        Eval(   8,   14),
        Eval(   1,    5),
    ],
    [
        Eval(   4,   -1),
        Eval(  11,    2),
        Eval( -11,    6),
        Eval( -14,   13),
        Eval( -14,    9),
        Eval( -17,    6),
        Eval(  18,   -1),
        Eval(  16,   -7),
    ],
    [
        Eval( -26,   17),
        Eval( -24,   21),
        Eval(   7,   18),
        Eval(  23,   12),
        Eval(  16,    2),
        Eval(  -9,   11),
        Eval( -17,   12),
        Eval( -15,    6),
    ],
    [
        Eval( -14,   -6),
        Eval( -12,   -3),
        Eval(  -2,   -3),
        Eval(  -7,    9),
        Eval( -10,    9),
        Eval(  -4,   -2),
        Eval(  17,    2),
        Eval(  32,   24),
    ],
    [
        Eval(  44,  -44),
        Eval(  52,   -4),
        Eval(  16,   16),
        Eval( -59,   26),
        Eval( -20,   13),
        Eval( -60,   24),
        Eval(  36,   -4),
        Eval(  20,  -33),
    ],
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(  15,   14),
    Eval(  11,    8),
    Eval(   6,    6),
    Eval(   4,   10),
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval( -64,  -71),
    Eval( -44,  -59),
    Eval( -32,  -43),
    Eval( -47,  -31),
    Eval( -26,  -38),
    Eval( -42,  -52),
    Eval( -30,  -59),
    Eval( -46,  -66),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(   0,    6),
    Eval( -21,   -2),
    Eval( -16,  -16),
    Eval( -35,  -20),
    Eval( -39,  -25),
    Eval( -28,  -12),
    Eval( -26,   -9),
    Eval( -31,   -1),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval( -16,  -34),
    Eval( -27,  -14),
    Eval( -26,   20),
    Eval(  21,   31),
    Eval(  47,   36),
    Eval(  57,   62),
];

#[rustfmt::skip]
const OPEN_FILE_EVAL: [Eval; 5] = [
    Eval( -10,   -3),
    Eval( -18,    8),
    Eval(  38,    0),
    Eval( -13,  -10),
    Eval( -49,  -22),
];

#[rustfmt::skip]
const SEMI_OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -9,    8),
    Eval( -15,   25),
    Eval(  22,    2),
    Eval(  -2,   26),
    Eval( -23,   17),
];

#[rustfmt::skip]
const PAWN_SHIELD_EVAL: [Eval; 5] = [
    Eval( -37,   -2),
    Eval( -23,  -15),
    Eval(  10,  -19),
    Eval(  48,   -9),
    Eval(  33,    9),
];

#[rustfmt::skip]
const PAWN_DEFENDED_EVAL: [Eval; 6] = [
    Eval(  22,   13),
    Eval(   3,    9),
    Eval(  -1,   21),
    Eval(  14,   22),
    Eval(  -7,   39),
    Eval( -49,   48),
];

#[rustfmt::skip]
const PAWN_ATTACKED_EVAL: [Eval; 6] = [
    Eval(  19,   21),
    Eval( -69,  -64),
    Eval( -63,  -63),
    Eval( -56,  -62),
    Eval( -56,  -38),
    Eval( -50,    8),
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
