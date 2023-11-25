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
    Eval( 112,  211),
    Eval( 322,  474),
    Eval( 351,  488),
    Eval( 470,  875),
    Eval(1282, 1273),
];

const BISHOP_PAIR_EVAL: Eval = Eval(29, 96);
const TEMPO: Eval = Eval(29, 9);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval( -18,  -22),
        Eval( -23,  -34),
        Eval(   3,  -36),
        Eval(  20,  -29),
        Eval(  39,   13),
        Eval(  58,   87),
        Eval(   0,    0),
    ],
    [
        Eval( -29,   -8),
        Eval( -13,   -7),
        Eval( -14,   -3),
        Eval(   6,   17),
        Eval(  24,   25),
        Eval(  70,    6),
        Eval(  51,    8),
        Eval( -67,   21),
    ],
    [
        Eval( -14,    1),
        Eval(  -1,    1),
        Eval(   3,    1),
        Eval(   4,    5),
        Eval(   6,   11),
        Eval(  42,    5),
        Eval(  -2,   13),
        Eval( -46,   26),
    ],
    [
        Eval( -17,   -6),
        Eval( -29,    0),
        Eval( -22,    4),
        Eval( -17,   19),
        Eval(   6,   26),
        Eval(  36,   24),
        Eval(  44,   34),
        Eval(  62,   17),
    ],
    [
        Eval(   2,  -13),
        Eval(  12,  -20),
        Eval(   2,    4),
        Eval(  -4,   30),
        Eval(  -1,   50),
        Eval(  29,   60),
        Eval(  15,   73),
        Eval(  50,   29),
    ],
    [
        Eval(  14,  -56),
        Eval(  -3,  -18),
        Eval( -35,    1),
        Eval( -39,   21),
        Eval( -13,   42),
        Eval(  38,   58),
        Eval(  56,   44),
        Eval(  61,    7),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -25,    1),
        Eval(  -4,   21),
        Eval( -15,    2),
        Eval(   4,   -9),
        Eval(   6,   -2),
        Eval(  21,   -4),
        Eval(  13,    9),
        Eval(  -8,  -20),
    ],
    [
        Eval( -15,    1),
        Eval(   1,    4),
        Eval(  -7,    6),
        Eval(   6,   14),
        Eval(   5,   12),
        Eval(   6,    0),
        Eval(   9,   12),
        Eval(   0,    4),
    ],
    [
        Eval(   5,    6),
        Eval(   8,    5),
        Eval(  -4,    6),
        Eval(  -7,   10),
        Eval(  -5,    6),
        Eval(  -8,    7),
        Eval(  14,    4),
        Eval(  15,   -1),
    ],
    [
        Eval( -17,   21),
        Eval( -16,   22),
        Eval(   0,   23),
        Eval(  12,   16),
        Eval(  13,    8),
        Eval(  -3,   16),
        Eval(   3,    9),
        Eval(   4,    1),
    ],
    [
        Eval(   1,   15),
        Eval(   1,   21),
        Eval(   3,   28),
        Eval(   1,   38),
        Eval(   1,   41),
        Eval(   8,   43),
        Eval(  24,   36),
        Eval(  39,   41),
    ],
    [
        Eval(  37,  -35),
        Eval(  34,   -2),
        Eval(  -1,   13),
        Eval( -56,   27),
        Eval( -10,    9),
        Eval( -58,   21),
        Eval(  24,   -8),
        Eval(  20,  -34),
    ],
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(   8,   11),
    Eval(   7,    8),
    Eval(   4,    4),
    Eval(   2,   11),
];

#[rustfmt::skip]
const BACKWARD_PAWN_EVAL: [Eval; 8] = [
    Eval(   0,    0),
    Eval(   2,   -7),
    Eval( -11,   -2),
    Eval(  -7,    1),
    Eval( -13,    1),
    Eval( -28,   -5),
    Eval( -19,  -12),
    Eval(   0,    0),
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval( -48,  -85),
    Eval( -25,  -58),
    Eval( -20,  -38),
    Eval( -28,  -27),
    Eval( -14,  -36),
    Eval( -27,  -49),
    Eval( -16,  -62),
    Eval( -27,  -82),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(   2,    9),
    Eval( -13,  -10),
    Eval( -11,  -13),
    Eval( -22,  -23),
    Eval( -21,  -23),
    Eval( -21,   -7),
    Eval( -15,   -7),
    Eval( -24,   12),
];

#[rustfmt::skip]
const PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval(  -3,  -14),
    Eval( -12,   -1),
    Eval( -10,   30),
    Eval(  17,   54),
    Eval(  47,   67),
    Eval(  58,   87),
];

#[rustfmt::skip]
const OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -3,   -9),
    Eval(  -7,    2),
    Eval(  32,    3),
    Eval( -11,   10),
    Eval( -51,  -10),
];

#[rustfmt::skip]
const SEMI_OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(  -4,   12),
    Eval(  -8,   29),
    Eval(  16,   17),
    Eval(   0,   17),
    Eval( -16,   18),
];

#[rustfmt::skip]
const PAWN_SHIELD_EVAL: [Eval; 5] = [
    Eval( -28,  -24),
    Eval(  -7,  -20),
    Eval(   8,    4),
    Eval(  24,   33),
    Eval(  22,   32),
];

#[rustfmt::skip]
const PAWN_DEFENDED_EVAL: [Eval; 6] = [
    Eval(  14,   11),
    Eval(   2,   11),
    Eval(   0,   18),
    Eval(   4,   20),
    Eval(  -6,   29),
    Eval( -46,   35),
];

#[rustfmt::skip]
const PAWN_ATTACKED_EVAL: [Eval; 6] = [
    Eval(   8,   20),
    Eval( -52,  -47),
    Eval( -50,  -59),
    Eval( -48,  -49),
    Eval( -41,  -11),
    Eval(   0,    0),
];

impl Eval {
    fn accum(&mut self, eval: Eval, count: i16) {
        *self = self.accum_to(eval, count);
    }

    const fn accum_to(self, eval: Eval, count: i16) -> Eval {
        Eval(self.0 + count * eval.0, self.1 + count * eval.1)
    }

    fn map<F>(self, mut f: F) -> Self
    where
        F: FnMut(i16) -> i16,
    {
        Self(f(self.0), f(self.1))
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

fn side_pawn_structure(
    side_pawns: Bitboard,
    enemy_pawns: Bitboard,
    side_attacks: Bitboard,
    enemy_attacks: Bitboard,
    side_to_move: Color,
) -> Eval {
    let mut eval = Eval(0, 0);
    let mut remaining_pawns = side_pawns;
    while remaining_pawns != 0 {
        let index = remaining_pawns.trailing_zeros();

        let file = index % 8;
        let file_mask = consts::A_FILE << (index % 8);
        let front_mask = if side_to_move == Color::White {
            consts::A_FILE << (index + 8)
        } else {
            file_mask ^ (consts::A_FILE << index)
        };

        if front_mask & side_pawns != 0 {
            eval.accum(DOUBLED_PAWN_EVAL[file as usize], 1);
        } else if (front_mask | (1 << index)) & (enemy_attacks | enemy_pawns) == 0 {
            let rank_index = if side_to_move == Color::White {
                index / 8 - 1
            } else {
                6 - index / 8
            };

            eval.accum(PASSED_PAWN_EVAL[rank_index as usize], 1);
        }

        let adjacent = ((file_mask << 1) & !consts::A_FILE) | ((file_mask & !consts::A_FILE) >> 1);
        if adjacent & side_pawns == 0 {
            eval.accum(ISOLATED_PAWN_EVAL[file as usize], 1);
        }

        let (stop, back_mask) = if side_to_move == Color::White {
            ((1 << (index + 8)), file_mask ^ (front_mask << 8))
        } else {
            ((1 << (index - 8)), file_mask ^ (front_mask >> 8))
        };

        if enemy_attacks & stop != 0 && side_attacks & back_mask == 0 {
            eval.accum(BACKWARD_PAWN_EVAL[file as usize], 1);
        }

        remaining_pawns &= remaining_pawns - 1;
    }

    eval
}

fn pawn_structure(white_pawns: Bitboard, black_pawns: Bitboard) -> Eval {
    let mut eval = Eval(0, 0);
    let white_attacks =
        ((white_pawns << 9) & !consts::A_FILE) | ((white_pawns & !consts::A_FILE) << 7);
    let black_attacks =
        ((black_pawns >> 7) & !consts::A_FILE) | ((black_pawns & !consts::A_FILE) >> 9);

    eval.accum(
        side_pawn_structure(white_pawns, black_pawns, white_attacks, black_attacks, Color::White),
        1,
    );
    eval.accum(
        side_pawn_structure(black_pawns, white_pawns, black_attacks, white_attacks, Color::Black),
        -1,
    );

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

fn white_king_safety(king: Bitboard, pawns: Bitboard, phase: i16) -> Eval {
    let mut eval = Eval(0, 0);

    // Pawn Shield:
    // If the king is on 1st or 2nd rank and not in the middle two files,
    // then give a bonus for up to 3 pawns on the 2rd and 3rd ranks on the
    // same side of the board as the king.
    const QS_AREA: Bitboard = 0x0707;
    const KS_AREA: Bitboard = 0xE0E0;

    if king & KS_AREA != 0 {
        let pawn_count = (pawns & (KS_AREA << 8)).count_ones();
        eval.accum(
            PAWN_SHIELD_EVAL[pawn_count as usize].map(|x| (x * phase) >> 3),
            1,
        );
    } else if king & QS_AREA != 0 {
        let pawn_count = (pawns & (QS_AREA << 8)).count_ones();
        eval.accum(
            PAWN_SHIELD_EVAL[pawn_count as usize].map(|x| (x * phase) >> 3),
            1,
        );
    }

    eval
}

fn pawn_piece(pieces: &[[Bitboard; 6]; 2]) -> Eval {
    let mut eval = Eval(0, 0);

    let white_pawn_attacks =
        ((pieces[0][0] << 9) & !consts::A_FILE) | ((pieces[0][0] & !consts::A_FILE) << 7);

    let black_pawn_attacks =
        ((pieces[1][0] >> 7) & !consts::A_FILE) | ((pieces[1][0] & !consts::A_FILE) >> 9);

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

    // doubled, isolated, and passed pawns
    eval.accum(
        pawn_structure(board.pieces()[0][0], board.pieces()[1][0]),
        1,
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

    let white_phase = popcnt(board.pieces()[0][1])
        + popcnt(board.pieces()[0][2])
        + 2 * popcnt(board.pieces()[0][3])
        + 4 * popcnt(board.pieces()[0][4]);

    let black_phase = popcnt(board.pieces()[1][1])
        + popcnt(board.pieces()[1][2])
        + 2 * popcnt(board.pieces()[1][3])
        + 4 * popcnt(board.pieces()[1][4]);

    eval.accum(
        white_king_safety(board.pieces()[0][5], board.pieces()[0][0], black_phase),
        1,
    );

    eval.accum(
        white_king_safety(
            board.pieces()[1][5].swap_bytes(),
            board.pieces()[1][0].swap_bytes(),
            white_phase,
        ),
        -1,
    );

    eval.accum(pawn_piece(board.pieces()), 1);

    resolve(board, eval)
}
