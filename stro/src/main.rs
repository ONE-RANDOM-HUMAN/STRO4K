use std::io;

use stro::{game::{GameBuf, Game}, search::Search, position::{Board, Color}, movegen::{MoveBuf, gen_moves}};

fn uci_loop() {
    let mut buffer = GameBuf::uninit();
    let (game, start) = Game::startpos(&mut buffer);
    let mut search = Search::new(game);

    for line in io::stdin().lines() {
        let line = line.unwrap();
        let line = line.trim();
        if line.starts_with("ucinewgame") {
            // do nothing
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
                let position = Board::from_fen(line.trim_start_matches("fen").trim_start()).unwrap();

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
                    search.game().make_move(*moves.iter().find(|x| x.to_string() == mov).unwrap());
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
        }
    }

}

fn main() {
    // Assume the first line is uci
    let mut uci = String::new();
    io::stdin().read_line(&mut uci).unwrap();

    println!("uciok");
    println!("id name STRO");
    println!("id author ONE_RANDOM_HUMAN");


    uci_loop();
}
