pub mod threads;

use std::cmp;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::evaluate::{self, MAX_EVAL, MIN_EVAL};
use crate::game::{Game, GameBuf};
use crate::movegen::{gen_moves, MoveBuf};
use crate::moveorder::{self, HistoryTable, KillerTable};
use crate::position::{Board, Move};
use crate::tt::{Bound, TTData, TT};

pub struct Search<'a> {
    game: Game<'a>,
    nodes: u64,
    pub start: std::time::Instant,
    search_time: std::time::Duration,
    tt: TT,
    running: Arc<AtomicBool>,
    history: [HistoryTable; 2],
    ply: [PlyData; 6144],
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
    /// # Safety
    /// The tt must always be valid
    pub unsafe fn new(game: Game<'a>, tt: TT, running: Arc<AtomicBool>) -> Self {
        Self {
            game,
            nodes: 0,
            start: std::time::Instant::now(),
            search_time: std::time::Duration::from_secs(0),
            tt,
            running,
            history: [HistoryTable::new(), HistoryTable::new()],
            ply: [PlyData::new(); 6144],
        }
    }

    pub fn clear_tt(&mut self) {
        self.tt.clear();
    }

    pub fn new_game(&mut self) {
        // tt must be cleared seperately
        self.history[0].reset();
        self.history[1].reset();
        self.ply.fill(PlyData::new());
    }

    pub fn search(&mut self, time_ms: u32, _inc_ms: u32, print_info: bool) -> (Move, i32) {
        self.nodes = 0;
        self.search_time = std::time::Duration::from_millis(u64::from(time_ms / 30));

        self.ply[0].static_eval = evaluate::evaluate(self.game.position()) as i16;

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
            return (moves[0].mov, score);
        }

        let mut searched = 0;
        'a: for depth in 0.. {
            let mut alpha = MIN_EVAL;
            searched = 0;

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
                alpha = cmp::max(alpha, score);

                searched += 1;
            }

            moves.sort_by_key(|x| cmp::Reverse(x.score));

            if print_info {
                println!(
                    "info depth {} nodes {} nps {} score cp {} pv {}",
                    depth + 1,
                    self.nodes,
                    (self.nodes as f64 / self.start.elapsed().as_secs_f64()) as u64,
                    moves[0].score,
                    moves[0].mov,
                )
            }
        }

        moves[0..searched].sort_by_key(|x| cmp::Reverse(x.score));
        (moves[0].mov, moves[0].score)
    }

    pub fn alpha_beta(&mut self, mut alpha: i32, beta: i32, depth: i32, ply: usize) -> Option<i32> {
        // Check if should stop
        if self.nodes % 4096 == 0 && self.should_stop() {
            return None;
        }

        self.nodes += 1;

        if self.game.is_repetition() {
            return Some(0);
        }

        let mut buffer = MoveBuf::uninit();
        let moves = gen_moves(self.game.position(), &mut buffer);

        // Checkmate and stalemate
        let is_check = self.game.position().is_check();
        if !moves.iter().any(|&mov| self.game.is_legal(mov)) {
            return Some(if is_check { MIN_EVAL } else { 0 });
        }

        // Only check 50mr after it is known that it is not checkmate
        if self.game.position().fifty_moves() >= 100 {
            return Some(0);
        }

        // Check extension
        let depth = if is_check { depth + 1 } else { depth };

        let mut ordered_moves = 0;
        let mut hash = 0;

        if depth > 0 {
            // Probe tt
            hash = self.game.position().hash();

            'tt: {
                let Some(tt_data) = self.tt.load(hash) else { break 'tt };
                let best_move = tt_data.best_move();

                let Some(index) = moves.iter().position(|&x| x == best_move) else { break 'tt };
                if !self.game.is_legal(moves[index]) {
                    break 'tt;
                }

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

        let static_eval = evaluate::evaluate(self.game.position());
        self.ply[ply].static_eval = static_eval as i16;

        // Null Move Pruning
        if !self.ply[ply].no_nmp && depth >= 4 && beta - alpha == 1 && !is_check {
            let r: i32 = if depth >= 6 { 3 } else { 2 };
            unsafe {
                self.game.make_null_move();
            }

            // Don't do nmp on the next ply
            self.ply[ply + 1].no_nmp = true;
            let eval = self.alpha_beta(-beta, -beta + 1, depth - r - 1, ply + 1);
            self.ply[ply + 1].no_nmp = false;

            unsafe {
                self.game.unmake_move();
            }

            let eval = -eval?;

            if eval >= beta {
                return Some(eval);
            }
        }

        // Order the noisy moves
        ordered_moves +=
            moveorder::order_noisy_moves(self.game.position(), &mut moves[ordered_moves..]);

        let improving = ply >= 2 && static_eval > i32::from(self.ply[ply - 2].static_eval);

        // Futility pruning
        let f_prune = depth <= 3 && !is_check && beta - alpha == 1;

        const F_PRUNE_MARGIN: i32 = 320;
        let f_prune = f_prune && static_eval + cmp::max(1, depth + improving as i32) * F_PRUNE_MARGIN <= alpha;

        // Stand pat in qsearch
        let mut best_eval = if depth <= 0 { static_eval } else { MIN_EVAL };
        let mut best_move = None;
        let mut bound = Bound::Upper;

        if best_eval >= beta {
            return Some(best_eval);
        }

        if best_eval > alpha {
            bound = Bound::Exact;
            alpha = best_eval;
        }

        // first quiet, non-tt move
        let first_quiet = ordered_moves;

        for i in 0..moves.len() {
            if i == ordered_moves {
                if depth > 0 {
                    ordered_moves += moveorder::order_quiet_moves(
                        &mut moves[ordered_moves..],
                        self.ply[ply].kt,
                        &self.history[self.game.position().side_to_move() as usize],
                    );
                } else {
                    break;
                }
            }

            let mov = moves[i];
            if depth <= 0 {
                assert!(mov.flags().is_noisy(), "{mov:?}");
            }

            if f_prune && depth <= 0 {
                // Delta pruning
                const PIECE_VALUES: [i32; 5] = [256, 832, 832, 1344, 2496];

                let capture = self
                    .game
                    .position()
                    .get_piece(mov.dest(), self.game.position().side_to_move().other())
                    .map_or(0, |x| PIECE_VALUES[x as usize]);

                let promo = mov
                    .flags()
                    .promo_piece()
                    .map_or(0, |x| PIECE_VALUES[x as usize]);

                if static_eval + capture + promo + F_PRUNE_MARGIN <= alpha {
                    continue;
                }
            }

            unsafe {
                if !self.game.make_move(mov) {
                    continue; // the move was illegal
                }
            }

            let gives_check = self.game.position().is_check();

            if f_prune && !mov.flags().is_noisy() && !gives_check {
                unsafe {
                    self.game.unmake_move();
                }

                continue;
            }

            // PVS
            let eval = if best_move.is_none() || depth <= 0 {
                -search! { self, self.alpha_beta(-beta, -alpha, depth - 1, ply + 1) }
            } else {
                let lmr_depth = if depth >= 3
                    && i >= 3
                    && beta - alpha == 1
                    && !mov.flags().is_noisy()
                    && !is_check
                    && !gives_check
                {
                    cmp::max(1, depth - depth / 4 - (i / 8) as i32 - 1)
                } else {
                    depth - 1
                };

                let eval =
                    -search! { self, self.alpha_beta(-alpha - 1, -alpha, lmr_depth, ply + 1) };

                // Re-search
                if eval > alpha && (eval < beta || lmr_depth != depth - 1) {
                    -search! { self, self.alpha_beta(-beta, -alpha, depth - 1, ply + 1) }
                } else {
                    eval
                }
            };

            unsafe {
                self.game.unmake_move();
            }

            if eval > best_eval {
                best_move = Some(mov);
                best_eval = eval;
            }

            if eval >= beta {
                bound = Bound::Lower;
                if !mov.flags().is_noisy() {
                    self.ply[ply].kt.beta_cutoff(mov);
                    self.history[self.game.position().side_to_move() as usize]
                        .beta_cutoff(mov, depth);

                    // Decrease history of searched moves
                    #[allow(clippy::needless_range_loop)]
                    for i in first_quiet..i {
                        self.history[self.game.position().side_to_move() as usize]
                            .failed_cutoff(moves[i], depth);
                    }
                }

                break;
            }

            if eval > alpha {
                bound = Bound::Exact;
                alpha = eval;
            }
        }

        // Store tt if not in qsearch
        if let Some(mov) = best_move {
            if depth > 0 {
                self.tt
                    .store(hash, TTData::new(mov, bound, best_eval, depth, hash));
            }
        }

        Some(best_eval)
    }

    pub fn game(&mut self) -> &mut Game<'a> {
        &mut self.game
    }

    pub fn bench() {
        let mut buffer = GameBuf::uninit();
        let (game, start) = Game::startpos(&mut buffer);
        let tt = TT::new((16 * 1024 * 1024).try_into().unwrap());
        let mut search = unsafe { Search::new(game, tt.clone(), Arc::new(AtomicBool::new(true))) };
        search.search_time = std::time::Duration::MAX;

        // Same fens used in perft testing
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        ];

        let mut duration = std::time::Duration::ZERO;
        for fen in fens {
            tt.clear();
            search.new_game();

            unsafe {
                search.game.reset(&start);
                search.game.add_position(Board::from_fen(fen).unwrap());
            }

            let start = std::time::Instant::now();
            search.alpha_beta(MIN_EVAL, MAX_EVAL, 9, 0);

            duration += start.elapsed()
        }

        let nodes = search.nodes;
        let nps = (search.nodes as f64 / duration.as_secs_f64()) as u64;
        println!("{nodes} nodes {nps} nps");

        tt.dealloc();
    }

    fn should_stop(&self) -> bool {
        !self.running.load(Ordering::Relaxed) || self.start.elapsed() >= self.search_time
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct SearchMove {
    score: i32,
    mov: Move,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct PlyData {
    kt: KillerTable,
    static_eval: i16,
    no_nmp: bool,
}

impl PlyData {
    fn new() -> Self {
        Self {
            kt: KillerTable::new(),
            static_eval: 0,
            no_nmp: false,
        }
    }
}
