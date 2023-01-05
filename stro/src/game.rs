use crate::{
    movegen::{gen_moves, MoveBuf},
    position::{Board, Move},
};

// Enough space to fit longest possible chess game with 50-mr
pub type GameBuf = std::mem::MaybeUninit<[Board; 6144]>;

/// Required to get around lifetimes
pub struct GameStart<'a>(Game<'a>);

#[repr(transparent)]
#[derive(Debug)]
pub struct Game<'a> {
    // points to current position
    // first position must be startpos
    ptr: *mut Board,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Game<'a> {
    pub fn startpos(buf: &'a mut GameBuf) -> (Self, GameStart<'a>) {
        let ptr: *mut Board = buf.as_mut_ptr().cast();

        // SAFETY: `ptr` is valid because it points to the buffer
        unsafe {
            ptr.write(Board::STARTPOS);
        }

        (
            Self {
                ptr,
                phantom: std::marker::PhantomData,
            },
            GameStart(Self {
                ptr,
                phantom: std::marker::PhantomData,
            })
        )
    }

    pub fn from_position(buf: &'a mut GameBuf, position: Board) -> (Self, GameStart<'a>) {
        let (mut game, start) = Self::startpos(buf);

        // SAFETY: Only one position has been stored
        unsafe {
            game.add_position(position);
        }

        (game, start)
    }

    /// # Safety
    /// The total number of position stored must not exceed 6144
    pub unsafe fn add_position(&mut self, position: Board) {
        unsafe {
            self.ptr = self.ptr.add(1);
            self.ptr.write(position);
        }
    }

    /// # Safety
    /// Only the `GameStart` created with the `Game` can be used
    pub unsafe fn reset(&mut self, start: &GameStart<'a>) {
        self.ptr = start.0.ptr;
    }

    pub fn from_fen(buf: &'a mut GameBuf, fen: &'_ str) -> Option<(Self, GameStart<'a>)> {
        Board::from_fen(fen).map(|position| Self::from_position(buf, position))
    }

    /// # Safety
    /// The total number of position stored must not exceed 6144
    pub unsafe fn make_move(&mut self, mov: Move) -> bool {
        let mut board = unsafe { self.ptr.read() };

        if !board.make_move(mov) {
            return false;
        }

        unsafe {
            self.ptr = self.ptr.add(1);
            self.ptr.write(board);
        }

        true
    }

    /// # Safety
    /// At least one position must remain in the game after the unmake.
    pub unsafe fn unmake_move(&mut self) {
        unsafe {
            self.ptr = self.ptr.sub(1);
        }
    }

    pub fn position(&self) -> &Board {
        // SAFETY: This is always a valid pointer which cannot be
        // invalidated without using unsafe
        unsafe { &*self.ptr }
    }

    pub fn is_repetition(&self) -> bool {
        let position = self.position();
        if position.fifty_moves() == 0 {
            return false;
        }

        let mut ptr = self.ptr;

        let mut count = 1;
        loop {
            // SAFETY: The pointer is always valid because the first
            // position is startpos, so `fifty_moves == 0` and we exit
            // the loop.
            unsafe {
                ptr = ptr.sub(1);
                
                if (*ptr).repetition_ep(position) {
                    count += 1;
                    if count == 3 {
                        return true;
                    }
                }

                // Can't have repetition after a 50-mr reset
                if (*ptr).fifty_moves() == 0 {
                    return false;
                }
            }
        }
    }

    /// # Safety
    /// `depth` additional positions should not exceed 6144 stored positions
    pub unsafe fn perft(&mut self, depth: usize) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut buffer = MoveBuf::uninit();
        let moves = gen_moves(self.position(), &mut buffer);

        if depth == 1 {
            return moves.len() as u64;
        }

        let mut count = 0;
        for mov in moves {
            unsafe {
                let legal = self.make_move(*mov);
                debug_assert!(legal);
                count += self.perft(depth - 1);
                self.unmake_move();
            }
        }

        count
    }

    /// # Safety
    /// See `perft
    pub unsafe fn divide(&mut self, depth: usize) -> Vec<(Move, u64)> {
        if depth == 0 {
            return Vec::new()
        }

        let mut buffer = MoveBuf::uninit();
        let moves = gen_moves(self.position(), &mut buffer);
        let mut moves = moves
            .iter()
            .map(|&mov| (mov, 0))
            .collect::<Vec<_>>();

        if depth == 1 {
            for (_, count) in &mut moves {
                *count = 1;
            }

            return moves;
        }

        for (mov, count) in &mut moves {
            unsafe {
                let legal = self.make_move(*mov);
                debug_assert!(legal);

                *count += self.perft(depth - 1);
                self.unmake_move();
            }
        }

        moves
    }
}

#[cfg(test)]
mod tests {
    use super::{GameBuf, Game};


    #[test]
    fn perft_startpos() {
        let mut buffer = GameBuf::uninit();
        let mut position = Game::from_fen(
            &mut buffer,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        )
        .unwrap().0;

        unsafe {
            assert_eq!(position.perft(1), 20);
            assert_eq!(position.perft(2), 400);
            assert_eq!(position.perft(3), 8902);
            assert_eq!(position.perft(4), 197281);
            assert_eq!(position.perft(5), 4865609);

            #[cfg(not(debug_assertions))]
            assert_eq!(position.perft(6), 119060324);
            #[cfg(not(debug_assertions))]
            assert_eq!(position.perft(7), 3195901860);
        }
    }

    #[test]
    fn perft_kiwipete() {
        let mut buffer = GameBuf::uninit();
        let mut position = Game::from_fen(
            &mut buffer,
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap().0;

        unsafe {
            assert_eq!(position.perft(1), 48);
            assert_eq!(position.perft(2), 2039);
            assert_eq!(position.perft(3), 97862);
            assert_eq!(position.perft(4), 4085603);

            #[cfg(not(debug_assertions))]
            assert_eq!(position.perft(5), 193690690);
            #[cfg(not(debug_assertions))]
            assert_eq!(position.perft(6), 8031647685);
        }
    }

    #[test]
    fn perft_3() {
        let mut buffer = GameBuf::uninit();
        let mut position =
            Game::from_fen(&mut buffer, "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap().0;
        unsafe {
            assert_eq!(position.perft(1), 14);
            assert_eq!(position.perft(2), 191);
            assert_eq!(position.perft(3), 2812);
            assert_eq!(position.perft(4), 43238);
            assert_eq!(position.perft(5), 674624);
            assert_eq!(position.perft(6), 11030083);

            #[cfg(not(debug_assertions))]
            assert_eq!(position.perft(7), 178633661);
            #[cfg(not(debug_assertions))]
            assert_eq!(position.perft(8), 3009794393);
        }
    }

    #[test]
    fn perft_4() {
        let mut buffer = GameBuf::uninit();
        let mut position = Game::from_fen(
            &mut buffer,
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        )
        .unwrap().0;

        unsafe {
            assert_eq!(position.perft(1), 6);
            assert_eq!(position.perft(2), 264);
            assert_eq!(position.perft(3), 9467);
            assert_eq!(position.perft(4), 422333);

            #[cfg(not(debug_assertions))]
            assert_eq!(position.perft(5), 15833292);
            #[cfg(not(debug_assertions))]
            assert_eq!(position.perft(6), 706045033);
        }
    }

    #[test]
    fn perft_5() {
        let mut buffer = GameBuf::uninit();

        let mut position = Game::from_fen(
            &mut buffer,
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        )
        .unwrap().0;

        unsafe {
            assert_eq!(position.perft(1), 44);
            assert_eq!(position.perft(2), 1486);
            assert_eq!(position.perft(3), 62379);
            assert_eq!(position.perft(4), 2103487);

            #[cfg(not(debug_assertions))]
            assert_eq!(position.perft(5), 89941194);
        }
    }

    #[test]
    fn perft_6() {
        let mut buffer = GameBuf::uninit();

        let mut position = Game::from_fen(
            &mut buffer,
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        )
        .unwrap().0;

        unsafe {
            assert_eq!(position.perft(1), 46);
            assert_eq!(position.perft(2), 2079);
            assert_eq!(position.perft(3), 89890);
            assert_eq!(position.perft(4), 3894594);

            #[cfg(not(debug_assertions))]
            assert_eq!(position.perft(5), 164075551);
            #[cfg(not(debug_assertions))]
            assert_eq!(position.perft(6), 6923051137);
        }
    }
}
