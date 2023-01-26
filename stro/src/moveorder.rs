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

pub fn order_noisy_moves(position: &Board, moves: &mut [Move]) -> usize {
    // Sorts in order of:
    // promos and promo-captures by promo piece
    // regular captures
    // other moves
    moves.sort_by_key(|mov| std::cmp::Reverse(mov.flags.0));

    // find first non-promo move
    let promo = moves
        .iter()
        .position(|x| !x.flags.is_promo())
        .unwrap_or(moves.len());

    // find first quiet move
    let noisy = moves[promo..]
        .iter()
        .position(|x| !x.flags.is_capture())
        .map_or(moves.len(), |x| x + promo);

    moves[promo..noisy].sort_by(|&lhs, &rhs| {
        cmp_mvv(position, lhs, rhs).then_with(|| cmp_lva(position, lhs, rhs))
    });

    noisy
}

pub fn order_quiet_moves(
    mut moves: &mut [Move],
    kt: KillerTable,
    history: &[[i64; 64]; 64],
) -> usize {
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
    moves.sort_by_key(|mov| cmp::Reverse(history[mov.origin as usize][mov.dest as usize]));

    len
}

fn cmp_mvv(position: &Board, lhs: Move, rhs: Move) -> cmp::Ordering {
    // 0 is ep
    let lhs_v = position
        .get_piece(lhs.dest, position.side_to_move().other())
        .map_or(0, |x| x as u8);

    let rhs_v = position
        .get_piece(rhs.dest, position.side_to_move().other())
        .map_or(0, |x| x as u8);

    lhs_v.cmp(&rhs_v).reverse()
}

fn cmp_lva(position: &Board, lhs: Move, rhs: Move) -> cmp::Ordering {
    let lhs_a = position
        .get_piece(lhs.origin, position.side_to_move())
        .unwrap();
    let rhs_a = position
        .get_piece(rhs.origin, position.side_to_move())
        .unwrap();

    (lhs_a as u8).cmp(&(rhs_a as u8))
}
