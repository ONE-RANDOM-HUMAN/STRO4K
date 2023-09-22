use crate::consts;
use crate::movegen::{bishop_moves, knight_moves, queen_moves, rook_moves, MoveFn};
use crate::position::{Bitboard, Board, Color};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
struct Eval(i16, i16);

pub const MAX_EVAL: i32 = 128 * 256 - 1;
pub const MIN_EVAL: i32 = -MAX_EVAL;

// Material eval adjusted to average mobility
const MATERIAL_EVAL: [Eval; 5] = [
    Eval(328, 344),
    Eval(739, 655),
    Eval(823, 697),
    Eval(1162, 1297),
    Eval(2630, 2332),
];

const MOBILITY_EVAL: [Eval; 4] = [Eval(30, 21), Eval(22, 12), Eval(17, 4), Eval(11, 1)];

const BISHOP_PAIR_EVAL: Eval = Eval(95, 180);

#[rustfmt::skip]
const PST: [[Eval; 16]; 6] = [
    [
        Eval(-44,   -9),
        Eval(-82,   19),
        Eval(-12,    3),
        Eval( -4,  -54),
        Eval(-50,  -23),
        Eval(-15,  -52),
        Eval( -5,  -42),
        Eval( -5,  -66),
        Eval(  4,   21),
        Eval( 41,  -14),
        Eval( 79,  -34),
        Eval( 46,  -29),
        Eval(114,  104),
        Eval(112,   97),
        Eval( 91,   89),
        Eval( 31,   81),
    ],
    [
        Eval(-37,  -44),
        Eval(-36,  -43),
        Eval(-18,  -40),
        Eval(-33,  -25),
        Eval(-43,    4),
        Eval(-26,    2),
        Eval(-30,  -12),
        Eval( -1,   15),
        Eval( 50,   29),
        Eval( 74,   23),
        Eval( 40,   47),
        Eval( 84,   49),
        Eval( -5,   11),
        Eval(100,   18),
        Eval(107,   16),
        Eval( 33,   -5),
    ],
    [
        Eval( 26,  -28),
        Eval(-29,  -26),
        Eval(-27,  -15),
        Eval( 42,  -30),
        Eval( 17,   13),
        Eval(  9,   10),
        Eval(-15,    8),
        Eval( 18,  -11),
        Eval(  5,   21),
        Eval( 51,    8),
        Eval( 43,   19),
        Eval( 23,   27),
        Eval(-46,   16),
        Eval(-13,   16),
        Eval(  0,   11),
        Eval(-11,   -4),
    ],
    [
        Eval(-49,  -28),
        Eval(  9,  -33),
        Eval( -3,  -43),
        Eval(-42,  -47),
        Eval(-59,   16),
        Eval(-28,   14),
        Eval(-25,    4),
        Eval(  6,  -16),
        Eval( 31,   41),
        Eval( 56,   45),
        Eval( 70,   30),
        Eval(101,    6),
        Eval( 64,   49),
        Eval( 96,   54),
        Eval(104,   40),
        Eval(117,   29),
    ],
    [
        Eval(-17,  -78),
        Eval(  3,  -96),
        Eval( -8, -104),
        Eval(-31, -104),
        Eval(-41,   -8),
        Eval(-49,   36),
        Eval(-25,   17),
        Eval( 22,  -12),
        Eval(-38,    5),
        Eval(-38,   93),
        Eval( 36,   93),
        Eval( 79,   66),
        Eval(-43,   14),
        Eval( 17,   72),
        Eval( 85,   72),
        Eval(115,   18),
    ],
    [
        Eval( 80,  -40),
        Eval(-21,  -37),
        Eval(-67,  -36),
        Eval( 53,  -60),
        Eval( 26,   -5),
        Eval(-16,   18),
        Eval(-50,   22),
        Eval(-46,   -6),
        Eval( 92,   39),
        Eval( 79,   54),
        Eval( 75,   56),
        Eval( 67,   38),
        Eval(111,   15),
        Eval(107,   48),
        Eval( 93,   55),
        Eval( 90,   38),
    ],
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval(-120, -114),
    Eval( -59,  -79),
    Eval( -93,  -49),
    Eval( -82,  -45),
    Eval( -97,  -34),
    Eval( -69,  -64),
    Eval( -22,  -63),
    Eval( -97,  -97),
];

const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(-19, -11),
    Eval(-17, -8),
    Eval(-57, -23),
    Eval(-62, -43),
    Eval(-91, -40),
    Eval(-43, -33),
    Eval(-38, -18),
    Eval(-77, -24),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval(-29, -34),
    Eval(-59,  -8),
    Eval(-14,  37),
    Eval( 36,  41),
    Eval(110,  93),
    Eval(106,  98),
];

const OPEN_FILE_EVAL: Eval = Eval(72, -2);
const SEMI_OPEN_FILE_EVAL: Eval = Eval(44, -6);

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

/// Mirrored Quarter PSTs
/// Each entry in the pst represents a 2x2 square, and the values
/// are mirrored across the D/E file
fn side_pst(pieces: &[Bitboard; 6], row_mask: u8) -> Eval {
    let mut eval = Eval(0, 0);
    for (i, mut pieces) in pieces.iter().copied().enumerate() {
        while pieces != 0 {
            let piece_index = pieces.trailing_zeros();
            let index = ((piece_index / 2) & 0b11) | ((piece_index / 4) & 0b1100);

            eval.accum(PST[i][(index as u8 ^ row_mask) as usize], 1);
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

fn side_open_file(rook: Bitboard, side_pawns: Bitboard, enemy_pawns: Bitboard) -> Eval {
    let mut eval = Eval(0, 0);
    let mut file = consts::A_FILE;
    for _ in 0..8 {
        if (side_pawns | enemy_pawns) & file == 0 {
            eval.accum(OPEN_FILE_EVAL, popcnt(rook & file))
        } else if (side_pawns & file) == 0 {
            eval.accum(SEMI_OPEN_FILE_EVAL, popcnt(rook & file));
        }

        file <<= 1;
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
    eval.accum(side_pst(&board.pieces()[1], 0b1100), -1);

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
            board.pieces()[0][3],
            board.pieces()[0][0],
            board.pieces()[1][0],
        ),
        1,
    );

    eval.accum(
        side_open_file(
            board.pieces()[1][3],
            board.pieces()[1][0],
            board.pieces()[0][0],
        ),
        -1,
    );

    resolve(board, eval)
}
