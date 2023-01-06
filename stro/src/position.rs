use std::fmt;

use crate::{consts, movegen};

pub type Bitboard = u64;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Board {
    pieces: [[Bitboard; 6]; 2],
    colors: [Bitboard; 2],
    side_to_move: Color,
    fifty_moves: u8,
    ep: Option<Square>,
    castling: u8,
    padding: u64,
}

unsafe fn _size_check() {
    // SAFETY: This is never called
    unsafe {
        std::mem::transmute::<_, Board>([0_u8; 128]);
    }
}

impl Board {
    pub const STARTPOS: Board = Board {
        pieces: [
            [
                0x0000_0000_0000_FF00,
                0x0000_0000_0000_0042,
                0x0000_0000_0000_0024,
                0x0000_0000_0000_0081,
                0x0000_0000_0000_0008,
                0x0000_0000_0000_0010,
            ],
            [
                0x00FF_0000_0000_0000,
                0x4200_0000_0000_0000,
                0x2400_0000_0000_0000,
                0x8100_0000_0000_0000,
                0x0800_0000_0000_0000,
                0x1000_0000_0000_0000,
            ],
        ],
        colors: [0x0000_0000_0000_FFFF, 0xFFFF_0000_0000_0000],
        side_to_move: Color::White,
        fifty_moves: 0,
        ep: None,
        castling: 0b1111,
        padding: 0,
    };

    pub fn get_piece(&self, sq: Square, color: Color) -> Option<Piece> {
        for i in 0..6 {
            if sq.intersects(self.pieces[color as usize][i]) {
                return Piece::from_index(i as u8);
            }
        }

        None
    }

    /// Makes a pseudo-legal move and returns a boolean
    /// indicating if the move was legal
    pub fn make_move(&mut self, mov: Move) -> bool {
        let piece = self.get_piece(mov.origin, self.side_to_move).unwrap();

        let pieces = &mut self.pieces[self.side_to_move as usize];

        // move the piece
        pieces[piece as usize] ^= mov.origin.as_mask();

        let dest_piece = mov.flags.promo_piece().unwrap_or(piece);
        pieces[dest_piece as usize] ^= mov.dest.as_mask();

        // captures
        if mov.flags.is_nonep_capture() {
            let dest_piece = self.get_piece(mov.dest, self.side_to_move.other()).unwrap();
            self.pieces[self.side_to_move.other() as usize][dest_piece as usize] ^=
                mov.dest.as_mask();
        } else if mov.flags == MoveFlags::EN_PASSANT {
            // rank of origin, file of destination
            let captured_index = (mov.origin as u8 & 0b111000) | (mov.dest as u8 & 0b000111);
            self.pieces[self.side_to_move.other() as usize][0] ^= 1 << captured_index;
        }

        let pieces = &mut self.pieces[self.side_to_move as usize];
        let mut king_area = pieces[Piece::King as usize];
        if piece == Piece::King {
            // remove castling rights and set shift
            let shift = if self.side_to_move == Color::White {
                self.castling &= 0b1100;
                0
            } else {
                self.castling &= 0b0011;
                56
            };

            if mov.flags == MoveFlags::QUEENSIDE_CASTLE {
                pieces[Piece::Rook as usize] ^= 0b0000_1001 << shift;
                king_area = 0b0001_1100 << shift;
            } else if mov.flags == MoveFlags::KINGSIDE_CASTLE {
                pieces[Piece::Rook as usize] ^= 0b1010_0000 << shift;
                king_area = 0b0111_0000 << shift;
            }
        }

        // update colors
        for c in 0..2 {
            let mut color = 0;
            for p in 0..6 {
                color |= self.pieces[c][p]
            }

            self.colors[c] = color;
        }

        // check for illegal move
        if self.is_area_attacked(king_area) {
            return false;
        }

        // remove castling rights
        let moved = mov.origin.as_mask() | mov.dest.as_mask();
        if Square::A1.intersects(moved) {
            self.castling &= 0b1110;
        }

        if Square::H1.intersects(moved) {
            self.castling &= 0b1101;
        }

        if Square::A8.intersects(moved) {
            self.castling &= 0b1011;
        }

        if Square::H8.intersects(moved) {
            self.castling &= 0b0111;
        }

        // ep target halfway between origin and dest
        self.ep = (mov.flags == MoveFlags::DOUBLE_PAWN_PUSH)
            .then(|| Square::from_index((mov.origin as u8 + mov.dest as u8) / 2).unwrap());

        // set 50 move rule
        self.fifty_moves = if piece == Piece::Pawn || mov.flags.is_nonep_capture() {
            0
        } else {
            self.fifty_moves + 1
        };

        // set side to move
        self.side_to_move = self.side_to_move.other();

        true
    }

    pub fn white(&self) -> u64 {
        self.colors[0]
    }

    pub fn black(&self) -> u64 {
        self.colors[1]
    }

    pub fn pieces(&self) -> &[[u64; 6]; 2] {
        &self.pieces
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn colors(&self) -> &[u64; 2] {
        &self.colors
    }

    pub fn ep(&self) -> Option<Square> {
        self.ep
    }

    pub fn castling(&self) -> u8 {
        self.castling
    }

    pub fn fifty_moves(&self) -> u8 {
        self.fifty_moves
    }

    pub fn repetition_eq(&self, other: &Board) -> bool {
        self.pieces == other.pieces
            && self.side_to_move == other.side_to_move
            && self.ep == other.ep
            && self.castling == other.castling
    }

    pub fn is_check(&self) -> bool {
        self.is_area_attacked(self.pieces[self.side_to_move as usize][5])
    }

    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut parts = fen.split_ascii_whitespace();
        let mut position = Self {
            pieces: [[0; 6]; 2],
            colors: [0; 2],
            side_to_move: Color::White,
            fifty_moves: 0,
            ep: None,
            castling: 0,
            padding: 0,
        };

        let mut file = 0;
        let mut rank = 7;

        // Placement of pieces
        for c in parts.next()?.bytes() {
            if file > 8 {
                return None;
            }

            match c {
                b'P' => position.pieces[0][0] |= 1 << (rank * 8 + file),
                b'N' => position.pieces[0][1] |= 1 << (rank * 8 + file),
                b'B' => position.pieces[0][2] |= 1 << (rank * 8 + file),
                b'R' => position.pieces[0][3] |= 1 << (rank * 8 + file),
                b'Q' => position.pieces[0][4] |= 1 << (rank * 8 + file),
                b'K' => position.pieces[0][5] |= 1 << (rank * 8 + file),
                b'p' => position.pieces[1][0] |= 1 << (rank * 8 + file),
                b'n' => position.pieces[1][1] |= 1 << (rank * 8 + file),
                b'b' => position.pieces[1][2] |= 1 << (rank * 8 + file),
                b'r' => position.pieces[1][3] |= 1 << (rank * 8 + file),
                b'q' => position.pieces[1][4] |= 1 << (rank * 8 + file),
                b'k' => position.pieces[1][5] |= 1 << (rank * 8 + file),
                b'1'..=b'8' => {
                    file += c - b'0';
                    continue;
                }
                b'/' => {
                    if file != 8 || rank == 0 {
                        return None;
                    }

                    file = 0;
                    rank -= 1;
                    continue;
                }
                b' ' => {
                    if file != 8 || rank != 0 {
                        return None;
                    }

                    break;
                }
                _ => return None,
            };

            file += 1;
        }

        // Calculate colours
        for i in 0..6 {
            position.colors[0] |= position.pieces[0][i];
            position.colors[1] |= position.pieces[1][i];
        }

        position.side_to_move = match parts.next()?.as_bytes() {
            b"w" => Color::White,
            b"b" => Color::Black,
            _ => return None,
        };

        let castling = parts.next()?;
        position.castling = if castling == "-" {
            0b0000
        } else {
            if castling.len() > 4 {
                return None;
            }

            // Accept both regular and shredder fen castling.
            let mut castling_rights = 0b0000;
            for c in castling.bytes() {
                match c {
                    b'k' => castling_rights |= 0b1000,
                    b'q' => castling_rights |= 0b0100,
                    b'K' => castling_rights |= 0b0010,
                    b'Q' => castling_rights |= 0b0001,
                    _ => return None,
                }
            }
            castling_rights
        };

        let en_passant = parts.next()?;
        position.ep = if en_passant == "-" {
            None
        } else {
            Some(en_passant.parse::<Square>().ok()?)
        };

        position.fifty_moves = parts.next()?.parse::<u8>().ok()?;

        // Ignore full moves

        Some(position)
    }

    fn is_area_attacked(&self, area: Bitboard) -> bool {
        let enemy = self.pieces[self.side_to_move.other() as usize];
        let occ = self.colors[0] | self.colors[1];

        let attacks = if self.side_to_move == Color::White {
            ((enemy[0] >> 7) & !consts::A_FILE) | ((enemy[0] & !consts::A_FILE) >> 9)
        } else {
            ((enemy[0] << 9) & !consts::A_FILE) | ((enemy[0] & !consts::A_FILE) << 7)
        };

        if attacks & area != 0 {
            return true;
        }

        let move_fns = [
            movegen::knight_moves,
            movegen::bishop_moves,
            movegen::rook_moves,
            movegen::queen_moves,
            movegen::king_moves,
        ];

        for i in 1..6 {
            if move_fns[i - 1](enemy[i], occ) & area != 0 {
                return true;
            }
        }

        false
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[rustfmt::skip]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    #[rustfmt::skip]
    const STR_SQ: [&'static str; 64] = [
        "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
        "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
        "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
        "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
        "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
        "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
        "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
        "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
    ];

    pub const fn from_index(index: u8) -> Option<Square> {
        if index < 64 {
            // SAFETY: Squares in 0..64 are valid
            unsafe { std::mem::transmute(index) }
        } else {
            None
        }
    }

    pub const fn offset(self, offset: i8) -> Option<Square> {
        Self::from_index((self as u8).wrapping_add_signed(offset))
    }

    pub const fn as_mask(self) -> Bitboard {
        1 << self as u8
    }

    pub const fn intersects(self, bb: Bitboard) -> bool {
        bb & self.as_mask() != 0
    }
}

impl std::str::FromStr for Square {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::STR_SQ
            .iter()
            .position(|&x| x == s)
            .map_or(Err(()), |x| Self::from_index(x as u8).ok_or(()))
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(Self::STR_SQ[*self as usize])
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub const fn other(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    pub const fn from_index(index: u8) -> Option<Piece> {
        if index < 6 {
            // SAFETY: Pieces in 0..6 are valid
            unsafe { Some(std::mem::transmute(index)) }
        } else {
            None
        }
    }
}

#[repr(C, align(4))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
/// Move: 24 bytes, compressed to 16 bytes for tt
pub struct Move {
    pub origin: Square,
    pub dest: Square,
    pub flags: MoveFlags,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const PROMOS: [&str; 4] = ["n", "b", "r", "q"];
        let promo = self.flags.promo_piece().map_or("", |p| PROMOS[p as usize - 1]);
        write!(f, "{}{}{}", self.origin, self.dest, promo)
    }
}

/// Move flags
/// Format:
/// bit 0: double pawn push
/// bit 1: en passant
/// bit 2: queenside castle
/// bit 3: kingside castle
/// bit 4: capture (non ep)
/// bit 5: promo
/// bits 7-6: promo piece
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MoveFlags(pub u8);

impl MoveFlags {
    pub const NONE: MoveFlags = MoveFlags(0b0);
    pub const DOUBLE_PAWN_PUSH: MoveFlags = MoveFlags(0b1);
    pub const EN_PASSANT: MoveFlags = MoveFlags(0b10);
    pub const QUEENSIDE_CASTLE: MoveFlags = MoveFlags(0b100);
    pub const KINGSIDE_CASTLE: MoveFlags = MoveFlags(0b1000);
    pub const CAPTURE: MoveFlags = MoveFlags(0b1_0000);
    pub const PROMO: MoveFlags = MoveFlags(0b10_0000);

    pub const fn is_promo(self) -> bool {
        self.0 & Self::PROMO.0 != 0
    }

    pub const fn is_nonep_capture(self) -> bool {
        self.0 & Self::CAPTURE.0 != 0
    }

    pub const fn is_noisy(self) -> bool {
        self.0 & (Self::CAPTURE.0 | Self::PROMO.0 | Self::EN_PASSANT.0) != 0
    }

    pub fn promo_piece(self) -> Option<Piece> {
        self.is_promo()
            .then(|| Piece::from_index((self.0 >> 6) + 1).unwrap())
    }
}
