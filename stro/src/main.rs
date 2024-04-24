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

    // SPSA
    println!("option name MinAspiration type spin default 32 min 0 max 1024");
    println!("option name SNmp type spin default 80 min 0 max 1024");
    println!("option name FPrune type spin default 128 min 0 max 1024");
    println!("option name DeltaBase type spin default 96 min 0 max 1024");
    println!("option name DeltaImprove type spin default 40 min 0 max 1024");
    println!("option name LmrBase type string default 0.25");
    println!("option name LmrDepth type string default 0.25");
    println!("option name LmrMove type string default 0.128");
    println!("option name LmrImprove type string default -1.0");
    println!("option name NmpBase type string default 3.0");
    println!("option name NmpDepth type string default 0.25");
    println!("option name NmpStaticEval type string default 0.01");
    println!("option name NmpImprove type string default -0.5");
    println!("option name MinTimeFrac type string default 0.025");
    println!("option name MinIncFrac type string default 0.0");
    println!("option name MaxTimeFrac type string default 0.05");
    println!("option name MaxIncFrac type string default 0.5");

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
                },
                "threads" => search.set_threads(value.parse().unwrap()),
                "asm" => match &*value.to_ascii_lowercase() {
                    "true" => search.set_asm(true),
                    "false" => search.set_asm(false),
                    _ => (),
                },
                "minaspiration" => unsafe {
                    stro::search::MIN_ASPIRATION_WINDOW_SIZE = value.parse().unwrap()
                }
                "snmp" => unsafe {
                    stro::search::STATIC_NULL_MOVE_MARGIN = value.parse().unwrap()
                }
                "fprune" => unsafe {
                    stro::search::F_PRUNE_MARGIN = value.parse().unwrap()
                }
                "deltabase" => unsafe {
                    stro::search::DELTA_BASE = value.parse().unwrap()
                }
                "deltaimprove" => unsafe {
                    stro::search::DELTA_IMPROVING_BONUS = value.parse().unwrap()
                }
                "lmrbase" => unsafe {
                    stro::search::LMR_BASE = value.parse().unwrap()
                }
                "lmrdepth" => unsafe {
                    stro::search::LMR_DEPTH = value.parse().unwrap()
                }
                "lmrmove" => unsafe {
                    stro::search::LMR_MOVE = value.parse().unwrap()
                }
                "lmrimprove" => unsafe {
                    stro::search::LMR_IMPROVING = value.parse().unwrap()
                }
                "nmpbase" => unsafe {
                    stro::search::NMP_BASE = value.parse().unwrap()
                }
                "nmpdepth" => unsafe {
                    stro::search::NMP_DEPTH = value.parse().unwrap()
                }
                "nmpstaticeval" => unsafe {
                    stro::search::NMP_STATIC_EVAL = value.parse().unwrap()
                }
                "nmpimprove" => unsafe {
                    stro::search::NMP_IMPROVING = value.parse().unwrap()
                }
                "mintimefrac" => unsafe {
                    stro::search::MIN_TIME_FRACTION = value.parse().unwrap()
                }
                "minincfrac" => unsafe {
                    stro::search::MIN_INC_FRACTION = value.parse().unwrap()
                }
                "maxtimefrac" => unsafe {
                    stro::search::MAX_TIME_FRACTION = value.parse().unwrap()
                }
                "maxincfrac" => unsafe {
                    stro::search::MAX_INC_FRACTION = value.parse().unwrap()
                }
                name => eprintln!("Unrecognised option: {name}"),
            }
        } else if line.starts_with("quit") {
            break;
        }
    }
}
