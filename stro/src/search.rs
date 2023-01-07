use std::cmp;

use crate::evaluate::{self, MAX_EVAL, MIN_EVAL};
use crate::game::{Game, GameBuf};
use crate::movegen::{gen_moves, MoveBuf};
use crate::position::Move;
use crate::tt::{Bound, TTData, TT};

pub struct Search<'a> {
    game: Game<'a>,
    nodes: u64,
    pub start: std::time::Instant,
    search_time: std::time::Duration,
    tt: TT,
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
        }
    }

    pub fn new_game(&mut self) {
        self.tt.clear()
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

        if moves.len() == 1 {
            let score = evaluate::evaluate(self.game().position());
            self.print_move(moves[0], score);
            return;
        }

        let mut moves = moves
            .iter()
            .map(|&mov| SearchMove {
                score: MIN_EVAL,
                mov,
            })
            .collect::<Vec<_>>();

        'a: for depth in 0.. {
            let mut alpha = MIN_EVAL;

            for mov in &mut moves {
                unsafe {
                    self.game.make_move(mov.mov);
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

    pub fn alpha_beta(
        &mut self,
        mut alpha: i32,
        beta: i32,
        depth: i32,
        _ply: usize,
    ) -> Option<i32> {
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
        if moves.is_empty() {
            return Some(if is_check { evaluate::MIN_EVAL } else { 0 });
        }

        // Only check after it is known that it is not checkmate
        if self.game.position().fifty_moves() >= 100 {
            return Some(0);
        }

        // tt
        let hash = self.game.position().hash();
        if let Some(tt_data) = self.tt.load(hash) {
            let best_mov = tt_data.best_move();
            if let Some(index) = moves.iter().position(|&x| x == best_mov) {
                moves.swap(0, index);

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

        let static_eval = evaluate::evaluate(self.game.position());

        let mut best_eval = if depth <= 0 { static_eval } else { MIN_EVAL };
        let mut best_move = None;
        let mut bound = Bound::Upper;

        if best_eval >= beta {
            return Some(best_eval);
        }

        for mov in moves {
            // Quiescence
            if depth <= 0 && !mov.flags.is_noisy() {
                continue;
            }

            unsafe {
                self.game.make_move(*mov);
            }

            let eval = -search! { self, self.alpha_beta(-beta, -alpha, depth - 1, _ply + 1) };

            unsafe {
                self.game.unmake_move();
            }

            if eval > best_eval {
                best_move = Some(*mov);
                best_eval = eval;
            }

            if eval >= beta {
                bound = Bound::Lower;
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
        search.alpha_beta(MIN_EVAL, MAX_EVAL, 7, 0);

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
