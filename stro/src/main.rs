fn main() {
    use std::io;
    unsafe {
        stro::init();
    }

    match std::env::args().nth(1).as_deref() {
        Some("bench") => {
            // Openbench compat
            let depth = std::env::args().nth(2).map_or(7, |x| x.parse().unwrap());
            stro::search::Search::bench(depth);
            return;
        }
        Some("bench2") => {
            let depth = std::env::args().nth(2).map_or(8, |x| x.parse().unwrap());

            stro::search::Search::bench2(depth);
            return;
        }
        Some("perft") => {
            let depth = std::env::args().nth(2).map_or(6, |x| x.parse().unwrap());

            let mut buf = stro::game::GameBuf::zeroed();
            let (mut game, _) = stro::game::Game::startpos(&mut buf);

            let start = std::time::Instant::now();
            let perft = unsafe { game.perft(depth) };

            let duration = start.elapsed();

            println!("{perft} nodes, {}ms", duration.as_millis());
            println!("{} nps", perft as f64 / duration.as_secs_f64());
            return;
        }
        _ => (),
    }

    // Assume the first line is uci
    let mut uci = String::new();
    io::stdin().read_line(&mut uci).unwrap();

    println!("id name STRO");
    println!("id author ONE_RANDOM_HUMAN");

    // Openbench compat
    println!("option name Hash type spin default 16 min 1 max 131072");
    println!("option name Threads type spin default 1 min 1 max 128");
    println!("option name asm type check default false");

    println!("uciok");

    uci_loop();
}

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

            let Some(index) = line.find("moves") else {
                continue;
            };

            // make moves
            for mov in line[index + 6..].split_ascii_whitespace() {
                let mut buffer = MoveBuf::uninit();
                let moves = gen_moves(search.game().position(), &mut buffer);

                unsafe {
                    assert!(
                        search.make_move(
                            moves
                                .iter()
                                .map(|x| x.mov)
                                .find(|x| x.to_string() == mov)
                                .unwrap()
                        ),
                        "illegal move"
                    );
                }
            }
        } else if line.starts_with("go") {
            let start = stro::search::time_now();
            let (time_str, inc_str) = if search.game().position().side_to_move() == Color::White {
                ("wtime", "winc")
            } else {
                ("btime", "binc")
            };

            let mut parts = line.split_ascii_whitespace();
            let mut time = 0;
            let mut inc = 0;
            while let Some(command) = parts.next() {
                match command {
                    "wtime" | "btime" => {
                        let value = parts.next().unwrap().parse().unwrap();
                        if command == time_str {
                            time = value;
                        }
                    }
                    "winc" | "binc" => {
                        let value = parts.next().unwrap().parse().unwrap();
                        if command == inc_str {
                            inc = value;
                        }
                    }
                    "infinite" => {
                        // These will not overflow because time calculations are
                        // performed using u64
                        time = u32::MAX;
                        inc = u32::MAX;
                    }
                    _ => () // Ignore all other commands
                }
            }

            search.search(start, time, inc);
        } else if line.starts_with("setoption") {
            let name = line[line.find("name").unwrap() + 4..line.find("value").unwrap()]
                .trim()
                .to_ascii_lowercase();
            let value = line[line.find("value").unwrap() + 5..].trim();

            match &*name {
                "hash" => unsafe {
                    search.resize_tt_mb(value.parse().unwrap());
                },
                "threads" => search.set_threads(value.parse().unwrap()),
                "asm" => match &*value.to_ascii_lowercase() {
                    "true" => search.set_asm(true),
                    "false" => search.set_asm(false),
                    _ => (),
                },
                _ => (),
            }
        } else if line.starts_with("quit") {
            break;
        }
    }
}
