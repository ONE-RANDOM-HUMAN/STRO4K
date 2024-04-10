use crate::evaluate::MAX_EVAL;
use crate::position::{Board, Move, MovePlus, Piece};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct KillerTable([Option<Move>; 2]);
impl KillerTable {
    pub fn new() -> Self {
        Self([None; 2])
    }

    pub fn beta_cutoff(&mut self, mov: Move) {
        self.0[1] = self.0[0];
        self.0[0] = Some(mov);
    }

    pub fn index(&self, mov: Move) -> Option<usize> {
        self.0.iter().position(|&x| x == Some(mov))
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HistoryTable([i64; 64 * 64]);
impl HistoryTable {
    pub fn new() -> Self {
        HistoryTable([0; 64 * 64])
    }

    pub fn reset(&mut self) {
        self.0.fill(0);
    }

    pub fn get(&self, mov: Move) -> i64 {
        self.0[(mov.0.get() & 0x0FFF) as usize]
    }

    pub fn beta_cutoff(&mut self, mov: Move, depth: i32) {
        self.0[(mov.0.get() & 0x0FFF) as usize] += i64::from(depth) * i64::from(depth);
    }

    pub fn failed_cutoff(&mut self, mov: Move, depth: i32) {
        self.0[(mov.0.get() & 0x0FFF) as usize] -= i64::from(depth) * i64::from(depth);
    }
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self::new()
    }
}

/// SEE, but we only evaluate at most one defender and always
/// assume that we can capture it
pub fn simple_see(mov: Move, position: &Board) -> bool {
    // Same values as delta pruning, plus an eval for king
    const PIECE_VALUES: [i32; 6] = [114, 425, 425, 648, 1246, MAX_EVAL];
    let victim = position.get_piece(mov.dest(), position.side_to_move().other())
        .unwrap_or(Piece::Pawn);

    let mut gain = PIECE_VALUES[victim as usize];
    if let Some(piece) = position.area_attacked_by(mov.dest().as_mask()) {
        let attacker = position.get_piece(mov.origin(), position.side_to_move()).unwrap();
        gain += PIECE_VALUES[piece as usize] - PIECE_VALUES[attacker as usize];
    }

    gain >= 0
}

pub fn order_noisy_moves(position: &Board, moves: &mut [MovePlus]) -> usize {
    // Sorts in order of:
    // promos and promo-captures by promo piece and mvvlva
    // regular captures by mvvlva
    // other moves

    let mut noisy_count = 0;
    for MovePlus { mov, score } in moves.iter_mut() {
        *score = i16::from(mov.flags().0) << 8;

        if mov.flags().is_noisy() {
            noisy_count += 1
        }

        if mov.flags().is_capture() {
            let victim = position
                .get_piece(mov.dest(), position.side_to_move().other())
                .map_or(-1, |x| x as i16);

            let attacker = position
                .get_piece(mov.origin(), position.side_to_move())
                .unwrap() as i16;

            *score |= ((victim + 1) << 3) - attacker;
        }
    }

    noisy_count
}

pub fn order_quiet_moves(
    moves: &mut [MovePlus],
    kt: KillerTable,
    history: &HistoryTable,
) -> usize {
    for mov in &mut *moves {
        let mut score = (history.get(mov.mov) as f32).to_bits() as i32;

        // Invert for negatives
        if score.is_negative() {
            score ^= 0x7FFF_FFFF;
        }

        mov.score = (score >> 16) as i16;

        // killers
        if let Some(index) = kt.index(mov.mov) {
            mov.score = i16::MAX - index as i16
        }
    }

    moves.len()
}

