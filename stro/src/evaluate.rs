use crate::consts;
use crate::movegen::{MoveFn, bishop_moves, knight_moves, queen_moves, rook_moves};
use crate::position::{Bitboard, Board, Color};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
struct Eval(i16, i16);

pub const MAX_EVAL: i32 = 128 * 256 - 1;
pub const MIN_EVAL: i32 = -MAX_EVAL;

/// Values for use when we don't want seperate MG and EG values
pub const PIECE_VALUES: [i32; 6] = [146, 438, 438, 704, 1404, MAX_EVAL];

#[rustfmt::skip]
const MATERIAL_EVAL: [Eval; 5] = [
    Eval( 115,  178),
    Eval( 366,  488),
    Eval( 413,  487),
    Eval( 555,  854),
    Eval(1383, 1426),
];

const BISHOP_PAIR_EVAL: Eval = Eval(27, 95);
const TEMPO: Eval = Eval(30, 14);

#[rustfmt::skip]
const RANK_PST: [[Eval; 8]; 6] = [
    [
        Eval(   0,    0),
        Eval( -11,   -7),
        Eval( -21,  -18),
        Eval(   1,  -17),
        Eval(  18,   -6),
        Eval(  47,   19),
        Eval(  83,  119),
        Eval(   0,    0),
    ],
    [
        Eval( -27,    0),
        Eval( -13,    4),
        Eval( -14,   10),
        Eval(  18,   31),
        Eval(  39,   30),
        Eval(  74,   14),
        Eval(  64,   13),
        Eval( -81,   31),
    ],
    [
        Eval( -10,   13),
        Eval(   7,   10),
        Eval(  11,   13),
        Eval(  12,   15),
        Eval(  11,   22),
        Eval(  42,   17),
        Eval(  -1,   22),
        Eval( -44,   35),
    ],
    [
        Eval( -10,   15),
        Eval( -21,   21),
        Eval( -17,   26),
        Eval( -10,   40),
        Eval(  17,   42),
        Eval(  49,   33),
        Eval(  42,   37),
        Eval(  72,   30),
    ],
    [
        Eval(  19,    6),
        Eval(  27,   10),
        Eval(  12,   48),
        Eval(   7,   80),
        Eval(  12,   98),
        Eval(  45,   93),
        Eval(  18,  105),
        Eval(  59,   73),
    ],
    [
        Eval(  15,  -60),
        Eval(  -2,  -19),
        Eval( -39,    1),
        Eval( -43,   25),
        Eval( -20,   47),
        Eval(  33,   65),
        Eval(  69,   52),
        Eval( 105,    1),
    ],
];

#[rustfmt::skip]
const FILE_PST: [[Eval; 8]; 6] = [
    [
        Eval( -22,    8),
        Eval(  -3,   26),
        Eval( -17,    9),
        Eval(   2,   -3),
        Eval(   4,    7),
        Eval(  29,   -1),
        Eval(  14,   15),
        Eval(   1,  -15),
    ],
    [
        Eval(  -9,    0),
        Eval(   3,    9),
        Eval(  -4,   19),
        Eval(  13,   27),
        Eval(   7,   27),
        Eval(   2,   15),
        Eval(  13,   19),
        Eval(   9,    5),
    ],
    [
        Eval(  12,    8),
        Eval(  15,   12),
        Eval(   0,   19),
        Eval(   1,   22),
        Eval(   0,   22),
        Eval(  -5,   23),
        Eval(  20,   13),
        Eval(  20,    6),
    ],
    [
        Eval( -17,   36),
        Eval( -12,   37),
        Eval(   4,   38),
        Eval(  17,   30),
        Eval(  18,   22),
        Eval(  -1,   32),
        Eval(   7,   25),
        Eval(   7,   13),
    ],
    [
        Eval(  16,   29),
        Eval(  16,   46),
        Eval(  13,   63),
        Eval(   9,   77),
        Eval(  11,   77),
        Eval(  19,   76),
        Eval(  40,   56),
        Eval(  54,   50),
    ],
    [
        Eval(  43,  -39),
        Eval(  37,   -2),
        Eval(  -2,   16),
        Eval( -67,   31),
        Eval( -14,   10),
        Eval( -61,   22),
        Eval(  24,  -11),
        Eval(  22,  -39),
    ],
];

#[rustfmt::skip]
const MOBILITY_EVAL: [Eval; 4] = [
    Eval(  13,    8),
    Eval(   8,    8),
    Eval(   5,    6),
    Eval(   4,    4),
];

#[rustfmt::skip]
const MOBILITY_ATTACK_EVAL: [[Eval; 4]; 4] = [
    [
        Eval(  -6,   27),
        Eval(  -1,   -8),
        Eval(  50,   32),
        Eval(  76,   11),
    ],
    [
        Eval(   3,   29),
        Eval(  19,   53),
        Eval(   3,    0),
        Eval(  47,   40),
    ],
    [
        Eval(   2,   34),
        Eval(  21,   34),
        Eval(  19,   41),
        Eval(  -5,   19),
    ],
    [
        Eval(   2,   14),
        Eval(   8,   23),
        Eval(  -5,   42),
        Eval( -22,   26),
    ],
];

#[rustfmt::skip]
const DOUBLED_PAWN_EVAL: [Eval; 8] = [
    Eval( -56,  -86),
    Eval( -33,  -51),
    Eval( -26,  -28),
    Eval( -38,  -16),
    Eval( -21,  -27),
    Eval( -32,  -37),
    Eval( -10,  -58),
    Eval( -34,  -80),
];

#[rustfmt::skip]
const ISOLATED_PAWN_EVAL: [Eval; 8] = [
    Eval(   5,    9),
    Eval( -14,  -14),
    Eval( -13,  -17),
    Eval( -30,  -24),
    Eval( -22,  -28),
    Eval( -20,  -10),
    Eval( -12,  -16),
    Eval( -21,   10),
];

#[rustfmt::skip]
const UNBLOCKED_PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval( -11,   -4),
    Eval( -17,    9),
    Eval( -10,   41),
    Eval(  21,   67),
    Eval(  47,  107),
    Eval( 102,  124),
];

#[rustfmt::skip]
const BLOCKED_PASSED_PAWN_EVAL: [Eval; 6] = [
    Eval( -13,    4),
    Eval( -27,   10),
    Eval( -20,   19),
    Eval(  17,   14),
    Eval(  49,   25),
    Eval(  16,   29),
];

#[rustfmt::skip]
const OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(   0,  -10),
    Eval(  -6,    3),
    Eval(  34,   -4),
    Eval( -11,   18),
    Eval( -55,   -6),
];

#[rustfmt::skip]
const SEMI_OPEN_FILE_EVAL: [Eval; 5] = [
    Eval(   0,    7),
    Eval(  -6,   22),
    Eval(  21,  -10),
    Eval(   3,    5),
    Eval( -13,   10),
];

#[rustfmt::skip]
const PAWN_SHIELD_EVAL: [[Eval; 5]; 2] = [
    [
        Eval( -33,  -22),
        Eval( -11,  -22),
        Eval( -14,    3),
        Eval( -16,   34),
        Eval( -35,   -8),
    ],
    [
        Eval(   0,    0),
        Eval(  -7,  -18),
        Eval(  11,    9),
        Eval(  25,   37),
        Eval(  14,   48),
    ],
];

#[rustfmt::skip]
const PAWN_DEFENDED_EVAL: [Eval; 6] = [
    Eval(  14,    7),
    Eval(   1,   10),
    Eval(   1,   15),
    Eval(  10,   16),
    Eval(  -2,   29),
    Eval( -50,   38),
];

#[rustfmt::skip]
const PAWN_ATTACKED_EVAL: [Eval; 6] = [
    Eval(   8,   31),
    Eval( -68,  -49),
    Eval( -51,  -78),
    Eval( -54,  -35),
    Eval( -54,   -9),
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

    let mut score = (i32::from(eval.0) * phase + i32::from(eval.1) * (24 - phase)) / 24;

    // Insufficient material
    if (0..=700).contains(&score) && board.pieces()[0][0] == 0
        || (-700..=0).contains(&score) && board.pieces()[1][0] == 0
    {
        score /= 4;
    }

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
        let knight_pawn = (pawns & 0x404000) != 0;
        eval.accum(
            PAWN_SHIELD_EVAL[knight_pawn as usize][pawn_count as usize].map(|x| (x * phase) >> 3),
            1,
        );
    } else if king & QS_AREA != 0 {
        let pawn_count = (pawns & (QS_AREA << 8)).count_ones();
        let knight_pawn = (pawns & 0x020200) != 0;
        eval.accum(
            PAWN_SHIELD_EVAL[knight_pawn as usize][pawn_count as usize].map(|x| (x * phase) >> 3),
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
