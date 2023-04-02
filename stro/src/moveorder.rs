use std::cmp;

use crate::position::{Board, Move};

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
        self.0[(mov.0.get() & 0x0FFF) as usize] -= i64::from(depth);
    }
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self::new()
    }
}

pub fn order_noisy_moves(position: &Board, moves: &mut [Move]) -> usize {
    // Sorts in order of:
    // promos and promo-captures by promo piece
    // regular captures
    // other moves

    insertion_sort_flags(moves);

    // increases performance by about 3% but loses guaranteed reproducibility
    // moves.sort_unstable_by_key(|mov| std::cmp::Reverse(mov.flags().0));

    // find first non-promo move
    let promo = moves
        .iter()
        .position(|x| !x.flags().is_promo())
        .unwrap_or(moves.len());

    // find first quiet move
    let noisy = moves[promo..]
        .iter()
        .position(|x| !x.flags().is_capture())
        .map_or(moves.len(), |x| x + promo);

    insertion_sort_by(&mut moves[promo..noisy], |lhs, rhs| {
        cmp_mvv(position, lhs, rhs).then_with(|| cmp_lva(position, lhs, rhs))
    });

    noisy
}

pub fn order_quiet_moves(mut moves: &mut [Move], kt: KillerTable, history: &HistoryTable) -> usize {
    // killers
    let len = moves.len();
    for mov in kt.0 {
        let Some(mov) = mov else { break; };

        if let Some(index) = moves.iter().position(|&x| x == mov) {
            moves.swap(0, index);
            moves = &mut moves[1..];
        }
    }

    // sort by history
    insertion_sort_by(moves, |lhs, rhs| {
        history.get(lhs).cmp(&history.get(rhs)).reverse()
    });

    len
}

pub fn insertion_sort_by<F>(moves: &mut [Move], mut cmp: F)
where
    F: FnMut(Move, Move) -> cmp::Ordering,
{
    for i in 1..moves.len() {
        let mov = moves[i];
        let mut j = i;
        while j > 0 {
            if cmp(moves[j - 1], mov) == cmp::Ordering::Greater {
                moves[j] = moves[j - 1];
            } else {
                break;
            }

            j -= 1
        }

        moves[j] = mov;
    }
}

/// Allows the use of a special comparison for flags
pub fn insertion_sort_flags(moves: &mut [Move]) {
    for i in 1..moves.len() {
        let mov = moves[i];
        let cmp = mov.0.get() & 0xF000;
        let mut j = i;
        while j > 0 {
            if moves[j - 1].0.get() < cmp {
                moves[j] = moves[j - 1];
            } else {
                break;
            }

            j -= 1
        }

        moves[j] = mov;
    }
}

fn cmp_mvv(position: &Board, lhs: Move, rhs: Move) -> cmp::Ordering {
    // 0 is ep
    let lhs_v = position
        .get_piece(lhs.dest(), position.side_to_move().other())
        .map_or(-1, |x| x as i8);

    let rhs_v = position
        .get_piece(rhs.dest(), position.side_to_move().other())
        .map_or(-1, |x| x as i8);

    lhs_v.cmp(&rhs_v).reverse()
}

fn cmp_lva(position: &Board, lhs: Move, rhs: Move) -> cmp::Ordering {
    let lhs_a = position
        .get_piece(lhs.origin(), position.side_to_move())
        .unwrap();
    let rhs_a = position
        .get_piece(rhs.origin(), position.side_to_move())
        .unwrap();

    (lhs_a as u8).cmp(&(rhs_a as u8))
}
