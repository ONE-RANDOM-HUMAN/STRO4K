use std::ops::Add;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};

use crate::game::{Game, GameBuf, GameStart};
use crate::position::{Board, Move};
use crate::search::Search;
use crate::tt::TT;

struct SearchThread {
    start: GameStart<'static>,
    search: Search<'static>,
    buffer: *mut GameBuf,
}

impl SearchThread {
    /// # Safety
    /// The TT must remain valid
    unsafe fn new(tt: TT, running: Arc<AtomicBool>) -> Self {
        let buffer = Box::into_raw(Box::new(GameBuf::uninit()));

        // SAFETY: Buffer is a valid pointer because it was just created
        // using `Box::into_raw`. It is not deallocated until drop is
        // called. `Game` and `Search` do not require the buffer in drop.
        let (game, start) = unsafe { Game::startpos(&mut *buffer) };

        // SAFETY: The tt must remain valid
        let search = unsafe { Search::new(game, tt, running) };

        Self {
            start,
            search,
            buffer,
        }
    }
}

impl Drop for SearchThread {
    fn drop(&mut self) {
        // SAFETY: `self.buffer` was created using `Box::into_raw` in `Self::new`
        unsafe {
            drop(Box::from_raw(self.buffer));
        }
    }
}

pub struct SearchThreads {
    threads: Vec<SearchThread>,
    main_thread: SearchThread,
}

impl SearchThreads {
    pub fn new(count: usize) -> Self {
        let tt = TT::new((16 * 1024 * 1024).try_into().unwrap());
        let running = Arc::new(AtomicBool::new(false));
        unsafe {
            Self {
                threads: std::iter::repeat_with(|| {
                    SearchThread::new(tt.clone(), Arc::clone(&running))
                })
                .take(count - 1)
                .collect(),
                main_thread: SearchThread::new(tt, running),
            }
        }
    }

    pub fn set_threads(&mut self, count: usize) {
        let search = &self.main_thread.search;
        self.threads.resize_with(count - 1, || unsafe {
            SearchThread::new(search.tt.clone(), Arc::clone(&search.running))
        });
    }

    pub fn game(&mut self) -> &Game {
        self.main_thread.search.game()
    }

    /// # Safety
    /// See `Game::make_move`
    pub unsafe fn make_move(&mut self, mov: Move) -> bool {
        unsafe { self.main_thread.search.game().make_move(mov) }
    }

    /// # Safety
    /// See `Game::add_position`
    pub unsafe fn add_position(&mut self, position: Board) {
        unsafe {
            self.main_thread.search.game().add_position(position);
        }
    }

    pub fn reset(&mut self) {
        // SAFETY: The `GameStart` was created with the game
        unsafe {
            self.main_thread
                .search
                .game()
                .reset(&self.main_thread.start);
        }
    }

    pub fn new_game(&mut self) {
        self.main_thread.search.clear_tt();
        self.main_thread.search.new_game();

        // Clear all threads
        for thread in &mut self.threads {
            thread.search.new_game();
        }
    }

    pub fn resize_tt_mb(&mut self, size: usize) {
        self.main_thread
            .search
            .tt
            .resize((size * 1024 * 1024).max(1).try_into().unwrap());

        // set the size for all threads
        for thread in &mut self.threads {
            thread.search.tt = self.main_thread.search.tt.clone();
        }
    }

    pub fn search(&mut self, start: time::Instant, time: u32, inc: u32) {
        // Get pointers to game of main thread
        let start_ptr = self.main_thread.start.as_mut_ptr();

        // SAFETY: Both pointers are valid and point to the same `GameBuf`
        let position_count = unsafe {
            (self.main_thread.search.game().position() as *const Board)
                .offset_from(start_ptr)
                .add(1) // position points to current pos, not one past the end
                .try_into()
                .unwrap()
        };

        let (mov, score) = thread::scope(|s| {
            self.main_thread
                .search
                .running
                .store(true, Ordering::Relaxed);
            for thread in &mut self.threads {
                // Copy the position
                let begin = thread.start.as_mut_ptr();

                unsafe {
                    begin.copy_from_nonoverlapping(start_ptr, position_count);
                    thread.search.game().set_ptr(begin.add(position_count - 1));
                }

                thread.search.start = start;
                s.spawn(|| {
                    thread.search.search(u32::MAX, u32::MAX, false);
                });
            }

            self.main_thread.search.start = start;
            let result = self.main_thread.search.search(time, inc, true);

            self.main_thread
                .search
                .running
                .store(false, Ordering::Relaxed);
            result
        });

        // not really centipawns, but no scaling to remain consistent
        // with a possible binary version.
        println!(
            "info nodes {} nps {} score cp {score}",
            self.main_thread.search.nodes,
            (self.main_thread.search.nodes as f64 / start.elapsed().as_secs_f64()) as u64
        );
        println!("bestmove {mov}")
    }
}
