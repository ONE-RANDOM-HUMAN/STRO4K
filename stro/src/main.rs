use std::io;

use stro::game::{Game, GameBuf};
use stro::movegen::{gen_moves, MoveBuf};
use stro::position::{Board, Color};
use stro::search::Search;

fn uci_loop() {
    let mut buffer = GameBuf::uninit();
    let (game, start) = Game::startpos(&mut buffer);
    let mut search = Search::new(game);

    for line in io::stdin().lines() {
        let line = line.unwrap();
        let line = line.trim();
        if line.starts_with("ucinewgame") {
            search.new_game();
        } else if line.starts_with("isready") {
            println!("readyok");
        } else if line.starts_with("position") {
            let line = line.trim_start_matches("position ").trim_start();

            unsafe {
                search.game().reset(&start);
            }

            if line.starts_with("startpos") {
                // do nothing
            } else if line.starts_with("fen") {
                // the fen parser doesn't care about what comes afterwards
                let position =
                    Board::from_fen(line.trim_start_matches("fen").trim_start()).unwrap();

                unsafe {
                    search.game().add_position(position);
                }
            }

            let Some(index) = line.find("moves") else { continue };

            // make moves
            for mov in line[index + 6..].split_ascii_whitespace() {
                let mut buffer = MoveBuf::uninit();
                let moves = gen_moves(search.game().position(), &mut buffer);

                unsafe {
                    search
                        .game()
                        .make_move(*moves.iter().find(|x| x.to_string() == mov).unwrap());
                }
            }
        } else if line.starts_with("go") {
            search.start = std::time::Instant::now();
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
            search.search(time, inc);
        } else if line.starts_with("setoption") {
            let name = line[line.find("name").unwrap() + 4..line.find("value").unwrap()]
                .trim()
                .to_ascii_lowercase();
            let value = line[line.find("value").unwrap() + 5..].trim();

            #[allow(clippy::single_match)]
            match &*name {
                "hash" => search.resize_tt_mb(value.parse().unwrap()),
                _ => (),
            }
        } else if line.starts_with("quit") {
            break;
        }
    }
}

fn main() {
    // Openbench compat
    if std::env::args().nth(1).map_or(false, |x| x == "bench") {
        Search::bench();
        return;
    }

    // Assume the first line is uci
    let mut uci = String::new();
    io::stdin().read_line(&mut uci).unwrap();

    println!("uciok");
    println!("id name STRO");
    println!("id author ONE_RANDOM_HUMAN");

    // Openbench compat
    println!("option name Hash type spin default 16 min 1 max 131072");
    println!("option name Threads type spin default 1 min 1 max 1");

    uci_loop();
}
