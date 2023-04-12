use std::ops::Add;
use std::sync::atomic::Ordering;
use std::thread;

use crate::game::{Game, GameBuf, GameStart};
use crate::position::{Board, Move};
use crate::search::RUNNING;
use crate::tt;

use super::{elapsed_nanos, Search, Time};

struct SearchThread {
    start: GameStart<'static>,
    search: Search<'static>,
    buffer: *mut GameBuf,
}

impl SearchThread {
    unsafe fn new() -> Self {
        let buffer = Box::into_raw(Box::new(GameBuf::uninit()));

        // SAFETY: Buffer is a valid pointer because it was just created
        // using `Box::into_raw`. It is not deallocated until drop is
        // called. `Game` and `Search` do not require the buffer in drop.
        let (game, start) = unsafe { Game::startpos(&mut *buffer) };

        let search = Search::new(game);

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
    asm: bool,
}

impl SearchThreads {
    /// # Safety
    /// The tt must not be accessed during the fuction call
    pub fn new(count: usize) -> Self {
        unsafe {
            tt::alloc((16 * 1024 * 1024).try_into().unwrap());

            Self {
                threads: std::iter::repeat_with(|| SearchThread::new())
                    .take(count - 1)
                    .collect(),
                main_thread: SearchThread::new(),
                asm: false,
            }
        }
    }

    pub fn set_threads(&mut self, count: usize) {
        self.threads
            .resize_with(count - 1, || unsafe { SearchThread::new() });
    }

    pub fn set_asm(&mut self, value: bool) {
        self.asm = value;
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
        tt::clear();
        self.main_thread.search.new_game();

        // Clear all threads
        for thread in &mut self.threads {
            thread.search.new_game();
        }
    }

    /// # Safety
    /// The tt must not be accessed during resize
    pub unsafe fn resize_tt_mb(&mut self, _size: u64) {
        unsafe {
            tt::alloc((_size * 1024 * 1024).max(1).try_into().unwrap());
        }
    }

    pub fn search(&mut self, start: Time, time: u32, inc: u32) {
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
            RUNNING.store(true, Ordering::Relaxed);

            for thread in &mut self.threads {
                // Copy the position
                let begin = thread.start.as_mut_ptr();

                unsafe {
                    begin.copy_from_nonoverlapping(start_ptr, position_count);
                    thread.search.game().set_ptr(begin.add(position_count - 1));
                }

                thread.search.start = start;

                s.spawn(|| {
                    #[cfg(feature = "asm")]
                    if self.asm {
                        thread.search.search_asm(u32::MAX, u32::MAX, false);
                    } else {
                        thread.search.search(u32::MAX, u32::MAX, false);
                    }

                    #[cfg(not(feature = "asm"))]
                    thread.search.search(u32::MAX, u32::MAX, false);
                });
            }

            self.main_thread.search.start = start;
            #[cfg(feature = "asm")]
            let result = if self.asm {
                let mov = self.main_thread.search.search_asm(time, inc, true);
                (mov, 0)
            } else {
                self.main_thread.search.search(time, inc, true)
            };

            #[cfg(not(feature = "asm"))]
            let result = self.main_thread.search.search(time, inc, true);

            RUNNING.store(false, Ordering::Relaxed);
            result
        });

        // not really centipawns, but no scaling to remain consistent
        // with a possible binary version.
        if !self.asm {
            println!(
                "info nodes {} nps {} score cp {score}",
                self.main_thread.search.nodes,
                (self.main_thread.search.nodes as f64
                    / (elapsed_nanos(&start) as f64 / 1_000_000_000.0)) as u64,
            );
        } else {
            println!(
                "info nodes {} nps {}",
                self.main_thread.search.nodes,
                (self.main_thread.search.nodes as f64
                    / (elapsed_nanos(&start) as f64 / 1_000_000_000.0)) as u64,
            );
        }

        println!("bestmove {mov}")
    }
}
