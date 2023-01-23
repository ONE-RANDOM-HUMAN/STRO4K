use std::cmp;

use crate::evaluate::{self, MAX_EVAL, MIN_EVAL};
use crate::game::{Game, GameBuf};
use crate::movegen::{gen_moves, MoveBuf};
use crate::moveorder;
use crate::position::Move;
use crate::tt::{Bound, TTData, TT};

pub struct Search<'a> {
    game: Game<'a>,
    nodes: u64,
    pub start: std::time::Instant,
    search_time: std::time::Duration,
    tt: TT,
    history: [[[i64; 64]; 64]; 2],
    kt: [moveorder::KillerTable; 6144],
}

/// Automatically unmakes move and returns when `None` is received
macro_rules! search {
    ($this:ident, $search_:expr) => {
        match $search_ {
            Some(x) => x,
            None => {
                unsafe {
                    $this.game.unmake_move();
                }

                return None;
            }
        }
    };
}

impl<'a> Search<'a> {
    pub fn new(game: Game<'a>) -> Self {
        Self {
            game,
            nodes: 0,
            start: std::time::Instant::now(),
            search_time: std::time::Duration::from_secs(0),
            tt: TT::new((16 * 1024 * 1024).try_into().unwrap()),
            history: [[[0; 64]; 64]; 2],
            kt: [moveorder::KillerTable::new(); 6144],
        }
    }

    pub fn new_game(&mut self) {
        self.tt.clear();
        self.history.fill([[0; 64]; 64]);
        self.kt.fill(moveorder::KillerTable::new());
    }

    pub fn resize_tt_mb(&mut self, size_in_mb: usize) {
        self.tt
            .resize((size_in_mb * 1024 * 1024).max(1).try_into().unwrap());
    }

    pub fn search(&mut self, time_ms: u32, _inc_ms: u32) {
        self.nodes = 0;
        self.search_time = std::time::Duration::from_millis(u64::from(time_ms / 30));

        let mut buffer = MoveBuf::uninit();
        let moves = gen_moves(self.game.position(), &mut buffer);

        let mut moves = moves
            .iter()
            .filter(|&&mov| self.game.is_legal(mov))
            .map(|&mov| SearchMove {
                score: MIN_EVAL,
                mov,
            })
            .collect::<Vec<_>>();

        if moves.len() == 1 {
            let score = evaluate::evaluate(self.game().position());
            self.print_move(moves[0].mov, score);
            return;
        }

        'a: for depth in 0.. {
            let mut alpha = MIN_EVAL;

            for mov in &mut moves {
                unsafe {
                    assert!(self.game.make_move(mov.mov));
                }

                let score = self.alpha_beta(MIN_EVAL, -alpha, depth, 1);

                // unmake the move before doing anything else
                unsafe {
                    self.game.unmake_move();
                }

                let score = match score {
                    Some(x) => -x,
                    None => break 'a,
                };

                mov.score = score;
                alpha = cmp::max(alpha, score)
            }

            moves.sort_by_key(|x| cmp::Reverse(x.score));

            println!(
                "info depth {} nodes {} nps {} score cp {} pv {}",
                depth + 1,
                self.nodes,
                (self.nodes as f64 / self.start.elapsed().as_secs_f64()) as u64,
                moves[0].score,
                moves[0].mov,
            )
        }

        moves.sort_by_key(|x| cmp::Reverse(x.score));
        self.print_move(moves[0].mov, moves[0].score)
    }

    pub fn alpha_beta(&mut self, mut alpha: i32, beta: i32, depth: i32, ply: usize) -> Option<i32> {
        if self.nodes % 4096 == 0 && self.should_stop() {
            return None;
        }

        self.nodes += 1;

        if self.game.is_repetition() {
            return Some(0);
        }

        let mut buffer = MoveBuf::uninit();
        let moves = gen_moves(self.game.position(), &mut buffer);

        let is_check = self.game.position().is_check();
        if !moves.iter().any(|&mov| self.game.is_legal(mov)) {
            return Some(if is_check { MIN_EVAL } else { 0 });
        }

        // Only check after it is known that it is not checkmate
        if self.game.position().fifty_moves() >= 100 {
            return Some(0);
        }

        // tt
        let mut ordered_moves = 0;
        let hash = self.game.position().hash();
        if let Some(tt_data) = self.tt.load(hash) {
            let best_mov = tt_data.best_move();
            if let Some(index) = moves.iter().position(|&x| x == best_mov) {
                if self.game.is_legal(moves[index]) {
                    moves.swap(0, index);
                    ordered_moves = 1;

                    if tt_data.depth() >= depth {
                        let eval = tt_data.eval();
                        match tt_data.bound() {
                            Bound::None => (),
                            Bound::Lower => {
                                if eval >= beta {
                                    return Some(eval);
                                }
                            }
                            Bound::Upper => {
                                if eval <= alpha {
                                    return Some(eval);
                                }
                            }
                            Bound::Exact => return Some(eval),
                        }
                    }
                }
            }
        }

        ordered_moves +=
            moveorder::order_noisy_moves(self.game.position(), &mut moves[ordered_moves..]);
        let static_eval = evaluate::evaluate(self.game.position());

        let mut best_eval = if depth <= 0 { static_eval } else { MIN_EVAL };
        let mut best_move = None;
        let mut bound = Bound::Upper;

        if best_eval >= beta {
            return Some(best_eval);
        }

        alpha = cmp::max(alpha, best_eval);

        for i in 0..moves.len() {
            if i >= ordered_moves {
                if depth > 0 {
                    ordered_moves += moveorder::order_quiet_moves(
                        &mut moves[ordered_moves..],
                        self.kt[ply],
                        &self.history[self.game.position().side_to_move() as usize],
                    );
                } else {
                    break;
                }
            }

            let mov = moves[i];

            // ignore quiet tt move in quiescence
            if depth <= 0 && !mov.flags.is_noisy() {
                continue;
            }

            unsafe {
                if !self.game.make_move(mov) {
                    continue;
                }
            }

            let eval = -search! { self, self.alpha_beta(-beta, -alpha, depth - 1, ply + 1) };

            unsafe {
                self.game.unmake_move();
            }

            if eval > best_eval {
                best_move = Some(mov);
                best_eval = eval;
            }

            if eval >= beta {
                bound = Bound::Lower;
                if !mov.flags.is_noisy() {
                    self.kt[ply].beta_cutoff(mov);
                    self.history[self.game.position().side_to_move() as usize]
                        [mov.origin as usize][mov.dest as usize] +=
                        i64::from(depth) * i64::from(depth);
                }

                break;
            }

            if eval > alpha {
                bound = Bound::Exact;
                alpha = eval;
            }
        }

        if let Some(mov) = best_move {
            self.tt
                .store(hash, TTData::new(mov, bound, best_eval, depth, hash));
        }

        Some(best_eval)
    }

    pub fn game(&mut self) -> &mut Game<'a> {
        &mut self.game
    }

    pub fn bench() {
        let mut buffer = GameBuf::uninit();
        let (game, _) = Game::startpos(&mut buffer);
        let mut search = Search::new(game);
        search.search_time = std::time::Duration::MAX;

        let start = std::time::Instant::now();
        search.alpha_beta(MIN_EVAL, MAX_EVAL, 8, 0);

        let nodes = search.nodes;
        let nps = (search.nodes as f64 / start.elapsed().as_secs_f64()) as u64;
        println!("{nodes} nodes {nps} nps");
    }

    fn should_stop(&self) -> bool {
        self.start.elapsed() >= self.search_time
    }

    fn print_move(&self, mov: Move, score: i32) {
        // not really centipawns, but no scaling to remain consistent
        // with a possible binary version.
        println!(
            "info nodes {} nps {} score cp {score}",
            self.nodes,
            (self.nodes as f64 / self.start.elapsed().as_secs_f64()) as u64
        );
        println!("bestmove {mov}")
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct SearchMove {
    score: i32,
    mov: Move,
}
