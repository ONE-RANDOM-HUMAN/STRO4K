use crate::consts;
use crate::movegen::{bishop_moves, knight_moves, queen_moves, rook_moves, MoveFn};
use crate::position::{Bitboard, Board, Color};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
struct Eval(i16, i16);

pub const MAX_EVAL: i32 = 128 * 256 - 1;
pub const MIN_EVAL: i32 = -MAX_EVAL;

/// Values for use when we don't want seperate MG and EG values
pub const PIECE_VALUES: [i32; 6] = [114, 425, 425, 648, 1246, MAX_EVAL];

#[rustfmt::skip]
const MATERIAL_EVAL: [Eval; 5] = [
    Eval( 110,  220),
    Eval( 319,  501),
    Eval( 358,  505),
    Eval( 490,  868),
    Eval(1312, 1301),
];

const BISHOP_PAIR_EVAL: Eval = Eval(29, 91);
const TEMPO: Eval = Eval(30, 15);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval( -14,  -31),
        Eval( -25,  -41),
        Eval(  -5,  -41),
        Eval(  12,  -29),
        Eval(  35,   11),
        Eval(  85,   94),
        Eval(   0,    0),
    ],
    [
        Eval( -44,  -11),
        Eval( -22,   -5),
        Eval( -16,   -1),
        Eval(  16,   18),
        Eval(  40,   18),
        Eval(  65,    6),
        Eval(  56,    7),
        Eval( -58,    8),
    ],
    [
        Eval( -22,    2),
        Eval(  -5,    0),
        Eval(   5,    1),
        Eval(  11,    2),
        Eval(  16,   10),
        Eval(  38,    7),
        Eval(  -1,   13),
        Eval( -45,   20),
    ],
    [
        Eval( -13,   -5),
        Eval( -25,    2),
        Eval( -20,    8),
        Eval( -13,   21),
        Eval(  10,   23),
        Eval(  37,   17),
        Eval(  32,   22),
        Eval(  53,   16),
    ],
    [
        Eval(   2,  -11),
        Eval(  10,  -16),
        Eval(  -1,   11),
        Eval(  -2,   37),
        Eval(   3,   55),
        Eval(  33,   57),
        Eval(  13,   61),
        Eval(  41,   27),
    ],
    [
        Eval(  13,  -56),
        Eval(  -1,  -18),
        Eval( -36,   -1),
        Eval( -37,   21),
        Eval( -12,   44),
        Eval(  38,   59),
        Eval(  58,   42),
        Eval(  57,    4),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -19,    2),
        Eval(  -6,   22),
        Eval( -18,    3),
        Eval(   3,  -10),
        Eval(   2,   -2),
        Eval(  24,   -6),
        Eval(  15,    8),
        Eval(  -5,  -20),
    ],
    [
        Eval(  -9,  -13),
        Eval(  -5,    0),
        Eval( -11,    8),
        Eval(   8,   16),
        Eval(   1,   15),
        Eval(   3,    3),
        Eval(  12,    5),
        Eval(   9,   -8),
    ],
    [
        Eval(   7,   -3),
        Eval(   5,    4),
        Eval(  -7,    9),
        Eval(  -5,   11),
        Eval( -10,   10),
        Eval(  -6,   10),
        Eval(  17,   -1),
        Eval(  20,   -9),
    ],
    [
        Eval( -18,   17),
        Eval( -15,   18),
        Eval(   2,   19),
        Eval(  14,   13),
        Eval(  15,    5),
        Eval(  -4,   14),
        Eval(   3,    9),
        Eval(   2,   -2),
    ],
    [
        Eval(  -1,    8),
        Eval(   1,   14),
        Eval(   2,   26),
        Eval(  -1,   37),
        Eval(   1,   40),
        Eval(   8,   41),
        Eval(  25,   32),
        Eval(  37,   37),
    ],
    [
        Eval(  37,  -38),
        Eval(  37,   -4),
        Eval(   3,   12),
        Eval( -56,   26),
        Eval( -11,   10),
        Eval( -57,   22),
        Eval(  24,   -7),
        Eval(  17,  -33),
    ],
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(  12,    8),
    Eval(   8,    7),
    Eval(   5,    7),
    Eval(   3,   10),
];

#[rustfmt::skip]
const MOBILITY_ATTACK_EVAL: [[Eval; 4]; 4] = [
    [
        Eval(  -4,   25),
        Eval(   2,   -9),
        Eval(  48,   30),
        Eval(  67,   17),
    ],
    [
        Eval(   2,   29),
        Eval(  20,   51),
        Eval(   4,    1),
        Eval(  44,   43),
    ],
    [
        Eval(   4,   32),
        Eval(  20,   34),
        Eval(  18,   40),
        Eval(  -4,   19),
    ],
    [
        Eval(   1,   19),
        Eval(   6,   27),
        Eval(  -6,   48),
        Eval( -25,   36),
    ],
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval( -54,  -85),
    Eval( -28,  -57),
    Eval( -22,  -38),
    Eval( -33,  -26),
    Eval( -14,  -34),
    Eval( -32,  -46),
    Eval( -12,  -60),
    Eval( -30,  -82),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(   3,    9),
    Eval( -15,  -10),
    Eval( -13,  -14),
    Eval( -28,  -19),
    Eval( -23,  -22),
    Eval( -22,   -8),
    Eval( -16,   -9),
    Eval( -24,   10),
];

#[rustfmt::skip]
const UNBLOCKED_PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval(  -8,  -14),
    Eval( -15,    1),
    Eval( -10,   32),
    Eval(  20,   56),
    Eval(  52,   76),
    Eval(  90,   96),
];

#[rustfmt::skip]
const BLOCKED_PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval(  -9,   -4),
    Eval( -16,   -4),
    Eval( -12,    7),
    Eval(  16,    4),
    Eval(  45,   -1),
    Eval(   8,   12),
];

#[rustfmt::skip]
const OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(   9,   -9),
    Eval(   8,    2),
    Eval(  32,   -3),
    Eval( -12,   10),
    Eval( -55,   -8),
];

#[rustfmt::skip]
const SEMI_OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(   9,    8),
    Eval(  10,   22),
    Eval(  18,   -8),
    Eval(   1,    9),
    Eval( -17,   11),
];

#[rustfmt::skip]
const PAWN_SHIELD_EVAL: [Eval; 5] = [
    Eval( -27,  -23),
    Eval(  -8,  -22),
    Eval(   9,    3),
    Eval(  25,   29),
    Eval(  19,   34),
];

#[rustfmt::skip]
const PAWN_DEFENDED_EVAL: [Eval; 6] = [
    Eval(  14,    6),
    Eval(   3,    9),
    Eval(   5,   12),
    Eval(  11,   17),
    Eval(  -2,   36),
    Eval( -43,   35),
];

#[rustfmt::skip]
const PAWN_ATTACKED_EVAL: [Eval; 6] = [
    Eval(   8,   23),
    Eval( -65,  -46),
    Eval( -54,  -74),
    Eval( -51,  -50),
    Eval( -50,  -19),
    Eval(   0,    0),
];

#[rustfmt::skip]
const SPACE: [Eval; 8] = [
    Eval(  15,   14),
    Eval(  28,   -2),
    Eval(  25,   -9),
    Eval(  14,   -4),
    Eval(  24,    0),
    Eval(  15,    4),
    Eval(  16,    9),
    Eval(  14,   10),
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

/// Mobility
/// Additionally gives a bonus for each pawn, knight, bishop, or rook
/// attacked which is not defended by an enemy pawn.
///
/// Including queens would be >20 bytes larger
fn side_mobility(
    side_pieces: &[Bitboard; 6],
    enemy_pieces: &[Bitboard; 6],
    occ: Bitboard,
    mask: Bitboard,
) -> Eval {
    const MOVE_FNS: [MoveFn; 4] = [knight_moves, bishop_moves, rook_moves, queen_moves];

    let mut eval = Eval(0, 0);
    for i in 0..4 {
        let mut side_pieces = side_pieces[i + 1];

        while side_pieces != 0 {
            let piece = side_pieces & side_pieces.wrapping_neg();
            let movement = MOVE_FNS[i](piece, occ) & mask;
            eval.accum(MOBILITY_EVAL[i], popcnt(movement));

            for (enemy_pieces, attack) in enemy_pieces[0..4].iter().zip(MOBILITY_ATTACK_EVAL[i]) {
                eval.accum(attack, popcnt(enemy_pieces & movement));
            }

            side_pieces &= side_pieces - 1;
        }
    }

    eval
}

fn side_pawn_structure(
    side_pawns: Bitboard,
    enemy_pawns: Bitboard,
    enemy_attacks: Bitboard,
    enemy_pieces: Bitboard,
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
            let (stop, rank_index) = if side_to_move == Color::White {
                (index + 8, index / 8 - 1)
            } else {
                (index - 8, 6 - index / 8)
            };

            if enemy_pieces & (1 << stop) != 0 {
                eval.accum(BLOCKED_PASSED_PAWN_EVAL[rank_index as usize], 1);
            } else {
                eval.accum(UNBLOCKED_PASSED_PAWN_EVAL[rank_index as usize], 1);
            }
        }

        let adjacent = ((file_mask << 1) & !consts::A_FILE) | ((file_mask & !consts::A_FILE) >> 1);
        if adjacent & side_pawns == 0 {
            eval.accum(ISOLATED_PAWN_EVAL[file as usize], 1);
        }

        remaining_pawns &= remaining_pawns - 1;
    }

    eval
}

fn pawn_structure(white_pawns: Bitboard, black_pawns: Bitboard, colors: &[Bitboard; 2]) -> Eval {
    let mut eval = Eval(0, 0);
    let white_attacks =
        ((white_pawns << 9) & !consts::A_FILE) | ((white_pawns & !consts::A_FILE) << 7);
    let black_attacks =
        ((black_pawns >> 7) & !consts::A_FILE) | ((black_pawns & !consts::A_FILE) >> 9);

    eval.accum(
        side_pawn_structure(
            white_pawns,
            black_pawns,
            black_attacks,
            colors[1],
            Color::White,
        ),
        1,
    );
    eval.accum(
        side_pawn_structure(
            black_pawns,
            white_pawns,
            white_attacks,
            colors[0],
            Color::Black,
        ),
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

fn white_space(white_pawns: Bitboard, white_space_pieces: Bitboard) -> Eval {
    let mut mask = white_pawns;
    mask |= mask >> 8;
    mask |= mask >> 16;
    mask |= mask >> 32;

    let mut eval = Eval(0, 0);
    let mut file = consts::A_FILE;

    #[allow(clippy::needless_range_loop)]
    for i in 0..8 {
        let count = popcnt(white_space_pieces & file & mask);
        eval.accum(SPACE[i], count);
        file <<= 1
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
    let white_attacks = ((board.pieces()[0][0] << 9) & !consts::A_FILE)
        | ((board.pieces()[0][0] & !consts::A_FILE) << 7);
    let black_attacks = ((board.pieces()[1][0] >> 7) & !consts::A_FILE)
        | ((board.pieces()[1][0] & !consts::A_FILE) >> 9);
    eval.accum(
        side_mobility(&board.pieces()[0], &board.pieces()[1], occ, !black_attacks),
        1,
    );
    eval.accum(
        side_mobility(&board.pieces()[1], &board.pieces()[0], occ, !white_attacks),
        -1,
    );

    // doubled, isolated, and passed pawns
    eval.accum(
        pawn_structure(board.pieces()[0][0], board.pieces()[1][0], board.colors()),
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

    // space
    eval.accum(white_space(board.pieces()[0][0], board.pieces()[0][1] | board.pieces()[0][2]), 1);
    eval.accum(white_space(board.pieces()[1][0].swap_bytes(), (board.pieces()[1][1] | board.pieces()[1][2]).swap_bytes()), 1);

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
