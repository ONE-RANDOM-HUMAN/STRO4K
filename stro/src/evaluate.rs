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
    Eval(249, 245),
    Eval(733, 736).accum_to(MOBILITY_EVAL[0], -4),
    Eval(929, 774).accum_to(MOBILITY_EVAL[1], -6),
    Eval(1266, 1455).accum_to(MOBILITY_EVAL[2], -7),
    Eval(2635, 2543).accum_to(MOBILITY_EVAL[3], -13),
];

const MOBILITY_EVAL: [Eval; 4] = [Eval(35, 23), Eval(27, 23), Eval(26, 20), Eval(18, 16)];
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval(-47, -115),
    Eval( 25,  -58),
    Eval( 24,  -83),
    Eval(-35,  -90),
    Eval(-34,  -88),
    Eval(-26,  -87),
    Eval( 22, -102),
    Eval(-48, -116),
];

const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(-26,  59),
    Eval(-26, -58),
    Eval(-26, -59),
    Eval(-34, -69),
    Eval(-38, -70),
    Eval(-27, -59),
    Eval(-27, -58),
    Eval(-38, -70),
];

const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval(  4,  54),
    Eval( 20,  84),
    Eval( 36, 116),
    Eval( 57, 172),
    Eval( 92, 204),
    Eval(108, 268),
];

const BISHOP_PAIR_EVAL: Eval = Eval(128, 128);

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

fn side_doubled_isolated_pawn(pawns: Bitboard) -> Eval {
    let mut eval = Eval(0, 0);
    let mut file = consts::A_FILE;
    for i in 0..8 {
        let file_count = popcnt(pawns & file);
        eval.accum(DOUBLED_PAWN_EVAL[i], file_count.max(1) - 1);

        let adjacent = ((file << 1) & !consts::A_FILE) | ((file & !consts::A_FILE) >> 1);

        if pawns & adjacent == 0 {
            eval.accum(ISOLATED_PAWN_EVAL[i], file_count);
        }

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

    let occ = board.white() | board.black();
    eval.accum(side_mobility(&board.pieces()[0], occ, consts::ALL), 1);
    eval.accum(side_mobility(&board.pieces()[1], occ, consts::ALL), -1);

    eval.accum(side_doubled_isolated_pawn(board.pieces()[0][0]), 1);
    eval.accum(side_doubled_isolated_pawn(board.pieces()[1][0]), -1);

    eval.accum(white_passed_pawn(board.pieces()[0][0], board.pieces()[1][0]), 1);
    eval.accum(white_passed_pawn(board.pieces()[1][0].swap_bytes(), board.pieces()[0][0].swap_bytes()), -1);
    resolve(board, eval)
}
