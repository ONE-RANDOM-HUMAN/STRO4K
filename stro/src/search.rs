pub mod threads;

use std::cmp;
use std::num::NonZeroU16;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::evaluate::{self, MAX_EVAL, MIN_EVAL};
use crate::game::{Game, GameBuf};
use crate::movegen::{MoveBuf, gen_moves};
use crate::moveorder::{self, HistoryTable, KillerTable};
use crate::position::{Board, Color, Move};
use crate::tt::{self, Bound, TTData};

const CORR_HIST_ENTRIES: usize = 1 << 16;
const CORR_HIST_SCALE_SHIFT: u32 = 9;
const CORR_HIST_SCALE: i32 = 1 << CORR_HIST_SCALE_SHIFT;
const CORR_HIST_MAX_WEIGHT: i32 = 1 << 5;
const CORR_HIST_MAX: i32 = 128;

#[unsafe(no_mangle)]
pub static RUNNING: AtomicBool = AtomicBool::new(false);

#[cfg(not(feature = "asm"))]
pub mod time {
    pub type Time = std::time::Instant;

    pub fn time_now() -> Time {
        std::time::Instant::now()
    }

    pub fn elapsed_nanos(time: &Time) -> u64 {
        time.elapsed().as_nanos() as u64
    }
}

#[cfg(feature = "asm")]
pub mod time {
    pub type Time = libc::timespec;

    pub fn time_now() -> Time {
        unsafe {
            let mut time = std::mem::MaybeUninit::uninit();
            assert_eq!(
                libc::clock_gettime(libc::CLOCK_MONOTONIC, time.as_mut_ptr()),
                0
            );
            time.assume_init()
        }
    }

    pub fn elapsed_nanos(start: &Time) -> u64 {
        let time = time_now();
        let elapsed = (time.tv_sec - start.tv_sec) * 1_000_000_000 + time.tv_nsec - start.tv_nsec;

        elapsed as u64
    }
}

pub use time::*;

#[cfg_attr(feature = "asm", repr(C))]
pub struct Search<'a> {
    game: Game<'a>,
    nodes: u64,
    start: Time,
    min_search_time: u64, // min search time in nanoseconds
    max_search_time: u64, // max search time in nanoseconds
    corrhist: [[i32; CORR_HIST_ENTRIES]; 2],
    ply: [PlyData; 12288],
    conthist_stack: [usize; 12288],
    history: [HistoryTable; 6 * 64 * 2],
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
    fn new(game: Game<'a>) -> Box<Self> {
        let mut result = Box::new_zeroed();
        let ptr: *mut Self = result.as_mut_ptr();
        unsafe {
            (&raw mut (*ptr).game).write(game);
            (&raw mut (*ptr).start).write(time_now());
            (&raw mut (*ptr).min_search_time).write(u64::MAX);
            (&raw mut (*ptr).max_search_time).write(u64::MAX);
            result.assume_init()
        }
    }

    pub fn new_game(&mut self) {
        // tt must be cleared seperately
        self.ply.fill(PlyData::new());
        self.conthist_stack.fill(0);

        for history in self.corrhist.iter_mut().flatten() {
            *history = 0;
        }

        for history in self.history.iter_mut() {
            history.reset();
        }
    }

    pub fn set_time(&mut self, time_ms: u32, inc_ms: u32) {
        self.min_search_time = (time_ms as u64) * 31796 + (inc_ms as u64) * 8946;
        self.max_search_time = (time_ms as u64) * 85387 + (inc_ms as u64) * 574673;
    }

    #[cfg(feature = "asm")]
    pub fn search_asm(&mut self, main_thread: bool, max_depth: i32) {
        unsafe { crate::asm::root_search_sysv(self, main_thread, max_depth) }
    }

    pub fn search(&mut self, main_thread: bool, max_depth: i32) -> SearchResult {
        self.nodes = 0;
        let mut best_move = None;
        let mut last_score = 0;

        let mut reached_depth = max_depth;
        'a: for depth in 1..=max_depth {
            let mut window = 21;
            let mut alpha = cmp::max(MIN_EVAL, last_score - window);
            let mut beta = cmp::min(MAX_EVAL, last_score + window);

            last_score = loop {
                let Some(score) = self.alpha_beta(alpha, beta, depth, 0) else {
                    reached_depth = depth - 1;
                    break 'a;
                };

                if score <= alpha && score != MIN_EVAL {
                    window *= 2;
                    alpha = cmp::max(MIN_EVAL, score - window);
                } else if score >= beta && score != MAX_EVAL {
                    window *= 2;
                    beta = cmp::min(MAX_EVAL, score + window);
                } else {
                    break score;
                }
            };

            best_move = self.ply[0].best_move;

            if main_thread {
                self.print_uci_info(depth, last_score);
            }

            if self.time_up(self.min_search_time) {
                reached_depth = depth;
                break 'a;
            }
        }

        SearchResult {
            best_move: best_move.unwrap(),
            score: last_score.try_into().unwrap(),
            depth: reached_depth,
        }
    }

    pub fn print_uci_info(&mut self, depth: i32, score: i32) {
        let mut pv = vec![self.ply[0].best_move.unwrap()];

        unsafe {
            assert!(self.game().make_move(pv[0]));
        }

        // Extract pv from transposition table
        loop {
            let hash = self.game.position().hash();
            let Some(next_move) = tt::load(hash).map(TTData::best_move) else {
                break;
            };

            if pv.len() >= depth as usize && !next_move.flags().is_noisy() {
                break;
            }

            let mut buffer = MoveBuf::uninit();
            if !gen_moves(self.game.position(), &mut buffer)
                .iter()
                .any(|x| x.mov == next_move)
            {
                break;
            }

            unsafe {
                if !self.game().make_move(next_move) {
                    break;
                }
            }

            pv.push(next_move);
        }

        for _ in 0..pv.len() {
            unsafe {
                self.game.unmake_move();
            }
        }

        print!(
            "info depth {} nodes {} nps {} score cp {} pv",
            depth,
            self.nodes,
            (self.nodes as f64 / (elapsed_nanos(&self.start) as f64 / 1_000_000_000.0)) as u64,
            score,
        );

        for mov in pv {
            print!(" {mov}");
        }

        println!();
    }

    pub fn alpha_beta(&mut self, mut alpha: i32, beta: i32, depth: i32, ply: usize) -> Option<i32> {
        self.nodes += 1;

        let mut buffer = MoveBuf::uninit();
        let moves = gen_moves(self.game.position(), &mut buffer);

        // Checkmate and stalemate
        let is_check = self.game.position().is_check();
        if let Some(mov) = moves.iter().find(|&mov| self.game.is_legal(mov.mov)) {
            self.ply[ply].best_move = Some(mov.mov);
        } else {
            return Some(if is_check { MIN_EVAL } else { 0 });
        }

        // Only check 50mr after it is known that it is not checkmate
        if self.game.position().fifty_moves() >= 100 {
            return Some(0);
        }

        // Check for repetition
        if self.game.is_repetition() {
            return Some(0);
        }

        // Check if should stop
        if !RUNNING.load(Ordering::Relaxed)
            || self.nodes % 4096 == 0 && self.time_up(self.max_search_time)
        {
            return None;
        }

        // Check extension
        let mut depth = if is_check { depth + 1 } else { depth };

        let mut ordered_moves = 0;
        let pv_node = beta - alpha != 1;

        let mut static_eval = {
            let eval = evaluate::evaluate(self.game.position());

            let pawn = self.corrhist[self.game.position().side_to_move() as usize]
                [self.game.position().pawn_hash() as usize % CORR_HIST_ENTRIES]
                >> CORR_HIST_SCALE_SHIFT;

            let white_non_pawn = self.corrhist[self.game.position().side_to_move() as usize]
                [self.game.position().non_pawn_hash(Color::White) as usize % CORR_HIST_ENTRIES]
                >> CORR_HIST_SCALE_SHIFT;

            let black_non_pawn = self.corrhist[self.game.position().side_to_move() as usize]
                [self.game.position().non_pawn_hash(Color::Black) as usize % CORR_HIST_ENTRIES]
                >> CORR_HIST_SCALE_SHIFT;

            let material = self.corrhist[self.game.position().side_to_move() as usize]
                [self.game.position().material_hash() as usize % CORR_HIST_ENTRIES]
                >> CORR_HIST_SCALE_SHIFT;

            eval + pawn + white_non_pawn + black_non_pawn + material
        };

        // Use non-tt static eval to ensure continuity
        self.ply[ply].static_eval = static_eval as i16;
        let improving = ply >= 2 && static_eval > i32::from(self.ply[ply - 2].static_eval);

        // Probe tt
        let hash = self.game.position().hash();
        let mut tt_success = false;

        'tt: {
            let Some(tt_data) = tt::load(hash) else {
                break 'tt;
            };
            let best_move = tt_data.best_move();

            let Some(index) = moves.iter().position(|&x| x.mov == best_move) else {
                break 'tt;
            };

            if !self.game.is_legal(moves[index].mov) {
                break 'tt;
            }

            if depth > 0 || moves[index].mov.flags().is_noisy() {
                moves.swap(0, index);
                moves[0].score = i16::MAX;
                ordered_moves = 1;
            }

            let eval = tt_data.eval();
            match tt_data.bound() {
                Bound::None => (),
                Bound::Lower => static_eval = cmp::max(static_eval, eval),
                Bound::Upper => static_eval = cmp::min(static_eval, eval),
                Bound::Exact => static_eval = eval,
            }

            if !pv_node && tt_data.depth() >= depth {
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

            tt_success = true;
        }

        if !tt_success && depth > 3 {
            depth -= 1;
        }

        self.ply[ply + 1].kt = KillerTable::new();
        self.conthist_stack[ply + 2] = 0;

        // Null Move Pruning
        if depth > 0 && !pv_node && !is_check && static_eval >= beta {
            // Static null move pruning
            if depth <= 7 {
                const STATIC_NULL_MOVE_MARGIN: i32 = 76;
                let margin = depth * STATIC_NULL_MOVE_MARGIN;

                if static_eval >= beta + margin {
                    return Some(beta);
                }
            }

            // Null move pruning
            if depth >= 3 {
                // Round towards -inf is fine
                let r = (557 + depth * 46 + 2 * (static_eval - beta) - 70 * improving as i32) >> 8;

                unsafe {
                    self.game.make_null_move();
                }

                let eval = self.alpha_beta(-beta, -beta + 1, depth - r - 1, ply + 1);

                unsafe {
                    self.game.unmake_move();
                }

                let eval = -eval?;

                if eval >= beta {
                    return Some(eval);
                }
            }
        }

        // Order the noisy moves
        ordered_moves +=
            moveorder::order_noisy_moves(self.game.position(), &mut moves[ordered_moves..]);

        // Futility pruning
        let f_prune = depth <= 7 && !is_check && !pv_node;

        const F_PRUNE_MARGIN: i32 = 83;
        let f_prune = f_prune
            && static_eval + cmp::max(1, depth + improving as i32) * F_PRUNE_MARGIN <= alpha;

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

        // If depth <= 0, there are no quiets anyway
        let mut quiets_to_go = if beta - alpha == 1 {
            let quiets = if improving {
                (70 + 13 * depth * depth) >> 4
            } else {
                (2 + 8 * depth * depth) >> 4
            };

            cmp::max(quiets, 1)
        } else {
            0 // This will become negative on decrement
        };

        for i in 0..moves.len() {
            if i == ordered_moves {
                if depth > 0 {
                    let hist_index = self.game.position().side_to_move() as usize;
                    let conthist_index = hist_index
                        + self.conthist_stack[ply + 1] / std::mem::size_of::<HistoryTable>();

                    let conthist2_index =
                        hist_index + self.conthist_stack[ply] / std::mem::size_of::<HistoryTable>();

                    ordered_moves += moveorder::order_quiet_moves(
                        &mut moves[ordered_moves..],
                        self.ply[ply].kt,
                        &self.history[hist_index],
                        &self.history[conthist_index],
                        &self.history[conthist2_index],
                    );
                } else {
                    break;
                }
            }

            let mov = {
                let mut best_index = i;
                for j in i + 1..moves.len() {
                    if moves[j].score > moves[best_index].score {
                        best_index = j;
                    }
                }

                moves.swap(i, best_index);
                moves[i].mov
            };

            if depth <= 7 {
                let see = self.game.position().see(mov);
                if see < cmp::min(0, depth * -65) && !pv_node && !is_check {
                    continue;
                }

                see
            } else {
                0
            };

            if depth <= 0 {
                assert!(mov.flags().is_noisy(), "{mov:?}");
            }

            unsafe {
                if !self.game.make_move(mov) {
                    continue; // the move was illegal
                }
            }

            let gives_check = self.game.position().is_check();

            if f_prune && !mov.flags().is_noisy() && !gives_check && best_eval > MIN_EVAL {
                unsafe {
                    self.game.unmake_move();
                }

                continue;
            }

            // Set conthist
            self.conthist_stack[ply + 2] = self.game.position().move_index() as usize
                * 2
                * std::mem::size_of::<HistoryTable>();

            // PVS
            let eval = if best_move.is_none() || depth <= 0 {
                -search! { self, self.alpha_beta(-beta, -alpha, depth - 1, ply + 1) }
            } else {
                let lmr_depth = if depth >= 2 && i >= 3 {
                    // Round towards -inf is fine
                    let reduction =
                        (136 + depth * 14 + i as i32 * 36 - improving as i32 * 169) / 256;

                    depth - reduction - 1
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
                #[allow(clippy::needless_range_loop)]
                if !mov.flags().is_noisy() {
                    self.ply[ply].kt.beta_cutoff(mov);
                    self.history[self.game.position().side_to_move() as usize]
                        .beta_cutoff(mov, depth);

                    // Decrease history of searched moves
                    for i in first_quiet..i {
                        self.history[self.game.position().side_to_move() as usize]
                            .failed_cutoff(moves[i].mov, depth);
                    }

                    {
                        let index = self.game.position().side_to_move() as usize
                            + self.conthist_stack[ply] / std::mem::size_of::<HistoryTable>();
                        let conthist = &mut self.history[index];

                        conthist.beta_cutoff(mov, depth);
                        for i in first_quiet..i {
                            conthist.failed_cutoff(moves[i].mov, depth);
                        }
                    }

                    {
                        let index = self.game.position().side_to_move() as usize
                            + self.conthist_stack[ply + 1] / std::mem::size_of::<HistoryTable>();
                        let conthist = &mut self.history[index];

                        conthist.beta_cutoff(mov, depth);
                        for i in first_quiet..i {
                            conthist.failed_cutoff(moves[i].mov, depth);
                        }
                    }
                }

                break;
            }

            if eval > alpha {
                bound = Bound::Exact;
                alpha = eval;
            }

            // Late moves pruning
            if !mov.flags().is_noisy() {
                quiets_to_go -= 1;
                if quiets_to_go == 0 {
                    break;
                }
            }
        }

        if let Some(mov) = best_move {
            let static_eval = self.ply[ply].static_eval as i32;
            self.ply[ply].best_move = best_move;

            if !is_check
                && depth > 0
                && !mov.flags().is_noisy()
                && (bound != Bound::Lower || best_eval >= static_eval)
                && (bound != Bound::Upper || best_eval <= static_eval)
            {
                let weight = cmp::min(depth * depth, CORR_HIST_MAX_WEIGHT);
                let diff = (best_eval - static_eval).clamp(-CORR_HIST_MAX, CORR_HIST_MAX);

                let entry = &mut self.corrhist[self.game().position().side_to_move() as usize]
                    [self.game().position().pawn_hash() as usize % CORR_HIST_ENTRIES];
                *entry = diff * weight
                    - ((*entry * (weight - CORR_HIST_SCALE)) >> CORR_HIST_SCALE_SHIFT);

                let entry = &mut self.corrhist[self.game().position().side_to_move() as usize]
                    [self.game().position().non_pawn_hash(Color::White) as usize % CORR_HIST_ENTRIES];
                *entry = diff * weight
                    - ((*entry * (weight - CORR_HIST_SCALE)) >> CORR_HIST_SCALE_SHIFT);

                let entry = &mut self.corrhist[self.game().position().side_to_move() as usize]
                    [self.game().position().non_pawn_hash(Color::Black) as usize % CORR_HIST_ENTRIES];
                *entry = diff * weight
                    - ((*entry * (weight - CORR_HIST_SCALE)) >> CORR_HIST_SCALE_SHIFT);

                let entry = &mut self.corrhist[self.game().position().side_to_move() as usize]
                    [self.game().position().material_hash() as usize % CORR_HIST_ENTRIES];

                *entry = diff * weight
                    - ((*entry * (weight - CORR_HIST_SCALE)) >> CORR_HIST_SCALE_SHIFT);
            }

            tt::store(hash, TTData::new(mov, bound, best_eval, depth, hash));
        }

        Some(best_eval)
    }

    pub fn game(&mut self) -> &mut Game<'a> {
        &mut self.game
    }

    pub fn bench(depth: i32) {
        let mut buffer = GameBuf::uninit();
        let (game, start) = Game::startpos(&mut buffer);
        let mut search = Search::new(game);

        unsafe {
            tt::alloc((32 * 1024 * 1024).try_into().unwrap());
        }

        RUNNING.store(true, Ordering::Relaxed);

        // Startpos, plus fens generated by taking every 10000th non-startpos
        // position from testing for 60fd95d419c57ea0d3b8ae4aedffc1d6e66112f1
        let fens = include_str!("../../fens.txt");

        let mut duration = std::time::Duration::ZERO;
        let mut nodes = 0;
        for fen in fens.lines() {
            tt::clear();
            search.new_game();

            unsafe {
                search.game.reset(&start);
                search.game.add_position(Board::from_fen(fen).unwrap());
            }

            let start = std::time::Instant::now();
            search.search(false, depth);

            nodes += search.nodes;
            duration += start.elapsed();
        }

        #[cfg(feature = "asm")]
        {
            let rust_node_count = nodes;

            for fen in fens.lines() {
                tt::clear();
                search.new_game();

                unsafe {
                    search.game.reset(&start);
                    search.game.add_position(Board::from_fen(fen).unwrap());
                }

                let start = std::time::Instant::now();
                search.search_asm(false, depth);

                nodes += search.nodes;
                duration += start.elapsed();
            }

            assert_eq!(rust_node_count, nodes - rust_node_count);
        }

        RUNNING.store(false, Ordering::Relaxed);

        let nps = (nodes as f64 / duration.as_secs_f64()) as u64;
        println!("{nodes} nodes {nps} nps");

        unsafe { tt::dealloc() }
    }

    /// Bench a using sequence of moves from a game to simulate the effects
    /// of the state retained between moves such as the TT and history tables.
    pub fn bench2(depth: i32) {
        let mut buffer = GameBuf::uninit();
        let (game, start) = Game::startpos(&mut buffer);
        let mut search = Search::new(game);

        unsafe {
            tt::alloc((32 * 1024 * 1024).try_into().unwrap());
        }

        RUNNING.store(true, Ordering::Relaxed);

        // game from testing for 60fd95d419c57ea0d3b8ae4aedffc1d6e66112f1
        let moves: Vec<_> = include_str!("../../game.txt")
            .split_ascii_whitespace()
            .collect();

        let mut duration = std::time::Duration::ZERO;
        let mut nodes = 0;

        {
            tt::clear();
            search.new_game();

            unsafe {
                search.game.reset(&start);
            }

            for moves in moves.chunks_exact(2) {
                let start = std::time::Instant::now();
                search.search(false, depth);

                duration += start.elapsed();
                nodes += search.nodes;

                unsafe {
                    assert!(search.make_move_str(moves[0]));
                    assert!(search.make_move_str(moves[1]));
                }
            }
        }

        #[cfg(feature = "asm")]
        {
            let rust_node_count = nodes;

            tt::clear();
            search.new_game();

            unsafe {
                search.game.reset(&start);
            }

            for moves in moves.chunks_exact(2) {
                let start = std::time::Instant::now();
                search.search_asm(false, depth);

                duration += start.elapsed();
                nodes += search.nodes;

                unsafe {
                    assert!(search.make_move_str(moves[0]));
                    assert!(search.make_move_str(moves[1]));
                }
            }

            assert_eq!(rust_node_count, nodes - rust_node_count);
        }

        RUNNING.store(false, Ordering::Relaxed);

        let nps = (nodes as f64 / duration.as_secs_f64()) as u64;
        println!("{nodes} nodes {nps} nps");

        unsafe { tt::dealloc() }
    }

    unsafe fn make_move_str(&mut self, mov: &str) -> bool {
        let mut buffer = MoveBuf::uninit();
        let moves = gen_moves(self.game().position(), &mut buffer);
        let Some(mov) = moves.iter().map(|x| x.mov).find(|x| x.to_string() == mov) else {
            return false;
        };

        unsafe { self.game.make_move(mov) }
    }

    fn time_up(&self, search_time: u64) -> bool {
        elapsed_nanos(&self.start) > search_time
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SearchResult {
    pub depth: i32,
    pub best_move: Move,
    pub score: i16,
}

impl SearchResult {
    fn to_u64(self) -> u64 {
        self.depth as u64 | ((self.best_move.0.get() as u64) << 32) | ((self.score as u64) << 48)
    }

    fn from_u64(v: u64) -> Option<Self> {
        Some(Self {
            depth: v as i32,
            best_move: Move(NonZeroU16::new((v >> 32) as u16)?),
            score: (v >> 48) as i16,
        })
    }
}

#[repr(C, align(8))]
#[derive(Clone, Copy, Debug)]
struct PlyData {
    kt: KillerTable,
    static_eval: i16,
    best_move: Option<Move>,
}

impl PlyData {
    fn new() -> Self {
        Self {
            kt: KillerTable::new(),
            static_eval: 0,
            best_move: None,
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn search_print_info_sysv(search: &mut Search, depth: i32, score: i32) {
    search.print_uci_info(depth, score);
}
