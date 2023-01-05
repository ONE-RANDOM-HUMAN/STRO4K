use std::cmp;

use crate::game::Game;
use crate::movegen::{MoveBuf, gen_moves};
use crate::evaluate::{self, MIN_EVAL};
use crate::position::Move;

pub struct Search<'a> {
    game: Game<'a>,
    nodes: u64,
    pub start: std::time::Instant,
    search_time: std::time::Duration,
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
        }
    }

    pub fn search(&mut self, time_ms: u32, _inc_ms: u32) {
        self.nodes = 0;
        self.search_time = std::time::Duration::from_millis(u64::from(time_ms / 30));

        let mut buffer = MoveBuf::uninit();
        let moves = gen_moves(self.game.position(), &mut buffer);

        if moves.len() == 1 {
            let score = evaluate::evaluate(self.game().position());
            self.print_move(moves[0], score)
        }

        let mut moves = moves
            .iter()
            .map(|&mov| SearchMove { score: MIN_EVAL, mov })
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
                    Some(x) => x,
                    None => break 'a,
                };

                mov.score = score;
                alpha = cmp::max(alpha, score)
            } 

            moves.sort_by_key(|x| cmp::Reverse(x.score));
        }

        
        moves.sort_by_key(|x| cmp::Reverse(x.score));
        self.print_move(moves[0].mov, moves[0].score)
    }

    pub fn alpha_beta(&mut self, mut alpha: i32, beta: i32, depth: i32, _ply: usize) -> Option<i32> {
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

        let static_eval = evaluate::evaluate(self.game.position());
        let mut best_eval = if depth <= 0 { static_eval } else { MIN_EVAL };

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

            let eval = -search!{ self, self.alpha_beta(-beta, -alpha, depth - 1, _ply + 1) };

            unsafe {
                self.game.unmake_move();
            }

            if eval >= beta {
                return Some(eval);
            }

            if eval > best_eval {
                best_eval = eval;

                if eval > alpha {
                    alpha = eval;
                }
            }
        }

        Some(best_eval)
    }

    pub fn game(&mut self) -> &mut Game<'a> {
        &mut self.game
    }

    fn should_stop(&self) -> bool {
        self.start.elapsed() >= self.search_time
    }

    fn print_move(&self, mov: Move, score: i32) {
        // not really centipawns, but no scaling to remain consistent
        // with a possible binary version.
        println!("info nodes {} nps {} score cp {score}", self.nodes, (self.nodes as f64 / self.start.elapsed().as_secs_f64()) as u64);
        println!("bestmove {mov}")
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct SearchMove {
    score: i32,
    mov: Move,
}