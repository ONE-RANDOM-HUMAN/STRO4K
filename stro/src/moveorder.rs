use crate::position::{Board, Move, MovePlus};

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

pub fn order_noisy_moves(position: &Board, moves: &mut [MovePlus]) -> usize {
    // Sorts in order of:
    // promos and promo-captures by promo piece and mvvlva
    // regular captures by mvvlva
    // other moves

    for MovePlus { mov, score } in moves.iter_mut() {
        *score = i16::from(mov.flags().0) << 8;

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

    insertion_sort_by_score(moves);
    moves
        .iter()
        .position(|x| !x.mov.flags().is_noisy())
        .unwrap_or(moves.len())
}

pub fn order_quiet_moves(
    moves: &mut [MovePlus],
    kt: KillerTable,
    history: &HistoryTable,
) -> usize {
    for mov in &mut *moves {
        let score = history.get(mov.mov) as f32;
        mov.score = (score.to_bits() >> 16) as i16;

        // killers
        if let Some(index) = kt.index(mov.mov) {
            mov.score = i16::MAX - index as i16
        }
    }

    insertion_sort_by_score(moves);
    moves.len()
}

fn insertion_sort_by_score(moves: &mut [MovePlus]) {
    for i in 1..moves.len() {
        let mov = moves[i];
        let mut j = i;
        while j > 0 {
            if moves[j - 1].score < mov.score {
                moves[j] = moves[j - 1];
            } else {
                break;
            }

            j -= 1
        }

        moves[j] = mov;
    }
}

