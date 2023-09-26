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
    Eval( 335,  340),
    Eval( 761,  691),
    Eval( 825,  715),
    Eval(1212, 1291),
    Eval(2557, 2363),
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
        Eval( -39,  -18),
        Eval( -39,  -46),
        Eval(   1,  -59),
        Eval(  39,  -33),
        Eval(  71,   32),
        Eval(  98,   91),
        Eval(   0,    0),
    ],
    [
        Eval( -45,  -46),
        Eval( -26,  -38),
        Eval( -33,  -20),
        Eval(   6,   30),
        Eval(  44,   46),
        Eval(  95,   27),
        Eval(  99,   20),
        Eval( -74,   25),
    ],
    [
        Eval( -17,  -32),
        Eval(   1,  -25),
        Eval(  13,   -3),
        Eval(  -3,   12),
        Eval(   1,   30),
        Eval(  70,   10),
        Eval(  -5,   10),
        Eval( -65,   30),
    ],
    [
        Eval( -28,  -37),
        Eval( -70,  -25),
        Eval( -55,   -6),
        Eval( -39,   20),
        Eval(  31,   32),
        Eval(  73,   27),
        Eval(  79,   49),
        Eval(  98,   25),
    ],
    [
        Eval(   3,  -90),
        Eval(   9,  -89),
        Eval( -24,   -6),
        Eval( -38,   61),
        Eval( -10,   84),
        Eval(  58,   72),
        Eval(  25,   86),
        Eval(  92,   12),
    ],
    [
        Eval(  -5,  -66),
        Eval( -46,  -14),
        Eval( -79,   13),
        Eval(  -8,   28),
        Eval(  58,   47),
        Eval(  84,   65),
        Eval(  92,   60),
        Eval( 100,   22),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -58,   25),
        Eval( -11,   42),
        Eval( -43,   29),
        Eval(   9,    1),
        Eval(   1,   15),
        Eval(  55,   -3),
        Eval(  46,   -1),
        Eval( -12,  -28),
    ],
    [
        Eval( -22,  -20),
        Eval(  -1,   -8),
        Eval(  -9,   -4),
        Eval(  12,    9),
        Eval(   4,    5),
        Eval(   7,  -18),
        Eval(  15,    8),
        Eval(   6,   -9),
    ],
    [
        Eval(  13,  -12),
        Eval(  18,   -2),
        Eval(  -7,   -6),
        Eval( -11,    4),
        Eval( -18,    3),
        Eval( -22,    1),
        Eval(  33,  -15),
        Eval(  34,  -29),
    ],
    [
        Eval( -31,   10),
        Eval( -22,   16),
        Eval(  16,   12),
        Eval(  39,    5),
        Eval(  35,   -7),
        Eval(   8,    2),
        Eval(  -2,    8),
        Eval( -27,   -7),
    ],
    [
        Eval(  -6,  -64),
        Eval( -15,  -20),
        Eval(  -6,    6),
        Eval(  -3,   22),
        Eval( -15,   41),
        Eval(  -9,   35),
        Eval(  39,    5),
        Eval(  65,   -9),
    ],
    [
        Eval(  50,  -34),
        Eval(  81,    2),
        Eval(  23,   15),
        Eval( -86,   34),
        Eval( -40,   15),
        Eval( -90,   25),
        Eval(  56,  -14),
        Eval(  22,  -39),
    ],
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval(-107, -109),
    Eval( -77,  -86),
    Eval( -68,  -64),
    Eval( -82,  -49),
    Eval( -64,  -48),
    Eval( -93,  -70),
    Eval( -69,  -77),
    Eval( -87,  -92),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(  -7,    0),
    Eval( -36,  -16),
    Eval( -42,  -32),
    Eval( -78,  -31),
    Eval( -65,  -49),
    Eval( -48,  -29),
    Eval( -38,  -28),
    Eval( -50,   -9),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval( -29,  -36),
    Eval( -47,  -11),
    Eval( -30,   37),
    Eval(  44,   51),
    Eval(  85,   55),
    Eval(  98,   91),
];

#[rustfmt::skip]
const OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -9,  -23),
    Eval( -19,    0),
    Eval(  68,   -6),
    Eval( -23,   29),
    Eval( -91,  -16),
];

#[rustfmt::skip]
const SEMI_OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -6,   11),
    Eval( -13,   40),
    Eval(  42,    1),
    Eval(   9,    8),
    Eval( -46,   30),
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

    resolve(board, eval)
}
