#[cfg(feature = "asm")]
struct PageAllocator;

#[cfg(feature = "asm")]
unsafe impl std::alloc::GlobalAlloc for PageAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        unsafe {
            self.alloc_zeroed(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        unsafe {
            libc::munmap(ptr.cast(), layout.size());
        }
    }

    unsafe fn alloc_zeroed(&self, layout: std::alloc::Layout) -> *mut u8 {
        assert!(layout.align() <= 4096);

        unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                layout.size(),
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
                -1,
                0
            ).cast()
        }
    }
}

// Workaround for glibc not working the use of clone.
#[global_allocator]
#[cfg(feature = "asm")]
static GLOBAL: PageAllocator = PageAllocator;

#[cfg(feature = "asm")]
fn main() {
    if std::env::args().nth(1).map_or(false, |x| x == "bench") {
        unsafe {
            stro::init();
        }

        stro::search::Search::bench();
        return;
    }

    unsafe {
        stro::asm::start_sysv();
    }
}

#[cfg(not(feature = "asm"))]
fn main() {
    use std::io;
    unsafe {
        stro::init();
    }

    // Openbench compat
    if std::env::args().nth(1).map_or(false, |x| x == "bench") {
        stro::search::Search::bench();
        return;
    }

    // Assume the first line is uci
    let mut uci = String::new();
    io::stdin().read_line(&mut uci).unwrap();

    println!("id name STRO");
    println!("id author ONE_RANDOM_HUMAN");

    // Openbench compat
    println!("option name Hash type spin default 16 min 1 max 131072");
    println!("option name Threads type spin default 1 min 1 max 128");

    println!("uciok");

    uci_loop();
}

#[cfg(not(feature = "asm"))]
fn uci_loop() {
    use std::io;

    use stro::movegen::{gen_moves, MoveBuf};
    use stro::position::{Board, Color};
    use stro::search::threads::SearchThreads;

    let mut search = SearchThreads::new(1);

    for line in io::stdin().lines() {
        let line = line.unwrap();
        let line = line.trim();
        if line.starts_with("ucinewgame") {
            search.new_game();
        } else if line.starts_with("isready") {
            println!("readyok");
        } else if line.starts_with("position") {
            let line = line.trim_start_matches("position ").trim_start();
            search.reset();

            if line.starts_with("startpos") {
                // do nothing
            } else if line.starts_with("fen") {
                // the fen parser doesn't care about what comes afterwards
                let position =
                    Board::from_fen(line.trim_start_matches("fen").trim_start()).unwrap();

                unsafe {
                    search.add_position(position);
                }
            }

            let Some(index) = line.find("moves") else { continue };

            // make moves
            for mov in line[index + 6..].split_ascii_whitespace() {
                let mut buffer = MoveBuf::uninit();
                let moves = gen_moves(search.game().position(), &mut buffer);

                unsafe {
                    assert!(
                        search.make_move(*moves.iter().find(|x| x.to_string() == mov).unwrap()),
                        "illegal move"
                    );
                }
            }
        } else if line.starts_with("go") {
            let start = stro::search::time_now();
            let (time, inc) = if search.game().position().side_to_move() == Color::White {
                ("wtime", "winc")
            } else {
                ("btime", "binc")
            };

            let mut parts = line.split_ascii_whitespace();

            #[allow(clippy::while_let_on_iterator)]
            while let Some(value) = parts.next() {
                if value == time {
                    break;
                }
            }

            let time = parts.next().unwrap().parse().unwrap();

            #[allow(clippy::while_let_on_iterator)]
            while let Some(value) = parts.next() {
                if value == inc {
                    break;
                }
            }

            let inc = parts.next().unwrap().parse().unwrap();
            search.search(start, time, inc);
        } else if line.starts_with("setoption") {
            let name = line[line.find("name").unwrap() + 4..line.find("value").unwrap()]
                .trim()
                .to_ascii_lowercase();
            let value = line[line.find("value").unwrap() + 5..].trim();

            match &*name {
                "hash" => unsafe {
                    search.resize_tt_mb(value.parse().unwrap());
                }
                "threads" => search.set_threads(value.parse().unwrap()),
                _ => (),
            }
        } else if line.starts_with("quit") {
            break;
        }
    }
}