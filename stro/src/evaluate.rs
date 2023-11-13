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
    Eval( 174,  244),
    Eval( 522,  571),
    Eval( 519,  547),
    Eval( 861, 1045),
    Eval(1954, 2207),
];

const BISHOP_PAIR_EVAL: Eval = Eval(60, 125);
const TEMPO: Eval = Eval(90, 47);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval( -27,  -10),
        Eval( -38,  -32),
        Eval(  10,  -40),
        Eval(  46,  -25),
        Eval(  77,   15),
        Eval(  76,  115),
        Eval(   0,    0),
    ],
    [
        Eval( -96,  -67),
        Eval( -45,  -46),
        Eval( -24,   -5),
        Eval(  10,   28),
        Eval(  45,   35),
        Eval( 122,    8),
        Eval(  75,  -21),
        Eval(-109,  -32),
    ],
    [
        Eval( -47,  -37),
        Eval( -18,  -28),
        Eval(  -7,  -16),
        Eval(  -3,   -6),
        Eval(   5,    0),
        Eval(  55,  -16),
        Eval(  -6,  -10),
        Eval(-120,    1),
    ],
    [
        Eval( -98,  -75),
        Eval(-120,  -66),
        Eval(-110,  -60),
        Eval( -97,  -31),
        Eval( -51,  -21),
        Eval(   2,  -25),
        Eval(  10,  -19),
        Eval(  -1,  -34),
    ],
    [
        Eval(-128, -128), // Tuner gave (-133, -170)
        Eval(-108, -128), // Tuner gave (-108, -160)
        Eval(-112,  -84),
        Eval(-107,  -18),
        Eval( -80,  -29),
        Eval( -25,  -92),
        Eval( -56, -105),
        Eval( -69, -128), // Tuner gave (-69, -151)
    ],
    [
        Eval(  32,  -87),
        Eval(   6,  -34),
        Eval( -43,    1),
        Eval( -49,   35),
        Eval( -60,   78),
        Eval( -36,   98),
        Eval(  55,   51),
        Eval(  70,  -25),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -34,    7),
        Eval(   4,   29),
        Eval( -28,   10),
        Eval(   5,  -11),
        Eval(   5,   -1),
        Eval(  28,  -12),
        Eval(  22,   -2),
        Eval( -12,  -29),
    ],
    [
        Eval( -72,  -58),
        Eval( -23,  -32),
        Eval( -17,    4),
        Eval(   0,    8),
        Eval(  -5,    7),
        Eval(   1,  -13),
        Eval( -12,  -20),
        Eval( -45,  -53),
    ],
    [
        Eval(  -9,  -32),
        Eval(   0,  -21),
        Eval( -25,  -14),
        Eval( -29,   -5),
        Eval( -27,  -11),
        Eval( -31,  -13),
        Eval(   5,  -26),
        Eval(   8,  -37),
    ],
    [
        Eval( -98,  -48),
        Eval( -97,  -43),
        Eval( -65,  -46),
        Eval( -47,  -50),
        Eval( -53,  -64),
        Eval( -81,  -54),
        Eval( -77,  -57),
        Eval( -72,  -68),
    ],
    [
        Eval(-124, -128), // Tuner gave (-124, -167)
        Eval(-108, -128),
        Eval( -98,  -89),
        Eval(-101,  -63),
        Eval(-104,  -61),
        Eval( -91, -110),
        Eval( -68, -128), // Tuner gave (-68, -153)
        Eval( -77, -128), // Tuner gave (-77, -165)
    ],
    [
        Eval(  76,  -57),
        Eval(  65,  -11),
        Eval(   3,   16),
        Eval( -88,   37),
        Eval( -16,   17),
        Eval( -96,   31),
        Eval(  39,   -9),
        Eval(  28,  -45),
    ],
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(   5,   -2),
    Eval(  11,    5),
    Eval(   4,    2),
    Eval(   7,  -33),
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval( -95, -101),
    Eval( -57,  -74),
    Eval( -42,  -42),
    Eval( -68,  -23),
    Eval( -27,  -35),
    Eval( -51,  -53),
    Eval( -25,  -76),
    Eval( -55,  -92),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(   5,    8),
    Eval( -22,  -14),
    Eval( -15,  -21),
    Eval( -41,  -25),
    Eval( -42,  -33),
    Eval( -34,  -10),
    Eval( -23,  -17),
    Eval( -40,    2),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval(  -6,  -24),
    Eval( -27,   -1),
    Eval( -27,   38),
    Eval(  23,   57),
    Eval(  72,   72),
    Eval(  76,  115),
];

#[rustfmt::skip]
const OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -8,  -13),
    Eval( -17,    2),
    Eval(  53,   -1),
    Eval( -11,   17),
    Eval( -72,  -21),
];

#[rustfmt::skip]
const SEMI_OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -6,    9),
    Eval( -17,   29),
    Eval(  28,    5),
    Eval(   3,  -11),
    Eval( -26,   24),
];

#[rustfmt::skip]
const PAWN_SHIELD_EVAL: [Eval; 5] = [
    Eval( -71,   36),
    Eval( -21,    0),
    Eval(  34,   -9),
    Eval(  81,   -5),
    Eval(  73,   20),
];

#[rustfmt::skip]
const PAWN_DEFENDED_EVAL: [Eval; 6] = [
    Eval(  27,   16),
    Eval(   6,   18),
    Eval(  -1,   27),
    Eval(  12,   22),
    Eval( -10,   25),
    Eval(-103,   68),
];

#[rustfmt::skip]
const PAWN_ATTACKED_EVAL: [Eval; 6] = [
    Eval(  28,   43),
    Eval( -94,  -96),
    Eval( -86, -112),
    Eval( -83, -108),
    Eval( -83,  -73),
    Eval(-102,   24),
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
