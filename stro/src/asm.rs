use crate::position::Bitboard;
use crate::position::{Board, Move};

use crate::game::Game;

#[allow(improper_ctypes)]
extern "C" {
    pub static mut SHIFTS: [u64; 8];

    pub fn gen_moves_sysv(board: &Board, moves: *mut Move) -> *mut Move;
    pub fn board_is_area_attacked_sysv(board: &Board, area: Bitboard) -> bool;
    pub fn game_is_repetition_sysv(game: &Game<'_>) -> bool;
    pub fn game_make_move_sysv(game: &Game<'_>, mov: u16) -> bool;
}

/// # Safety
/// No asm functions can be called concurrently
pub(crate) unsafe fn init() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe { SHIFTS = [8, 1, 9, 7, 17, 15, 10, 6] })
}

// movement functions using inline assembly so that non-sysv registers can be preserved
pub fn knight_moves(pieces: Bitboard, _occ: Bitboard) -> Bitboard {
    let result;
    unsafe {
        std::arch::asm!(
            "call knight_moves",
            in("r8") pieces,
            out("rax") result,
            out("xmm0") _,
            out("xmm1") _,
            out("xmm2") _,
            out("xmm3") _,
            out("xmm4") _,
            out("xmm5") _,
            out("xmm6") _,
            options(pure, readonly, nostack, raw),
        );
    }

    result
}

pub fn bishop_moves(pieces: Bitboard, occ: Bitboard) -> Bitboard {
    let result;
    unsafe {
        std::arch::asm!(
            "call bishop_moves",
            in("r8") pieces,
            in("r9") occ,
            out("rax") result,
            out("xmm0") _,
            out("xmm1") _,
            out("xmm2") _,
            out("xmm3") _,
            out("xmm4") _,
            out("xmm5") _,
            out("xmm6") _,
            options(pure, readonly, nostack, raw),
        );
    }

    result
}

pub fn rook_moves(pieces: Bitboard, occ: Bitboard) -> Bitboard {
    let result;
    unsafe {
        std::arch::asm!(
            "call rook_moves",
            in("r8") pieces,
            in("r9") occ,
            out("rax") result,
            out("xmm0") _,
            out("xmm1") _,
            out("xmm2") _,
            out("xmm3") _,
            out("xmm4") _,
            out("xmm5") _,
            out("xmm6") _,
            options(pure, readonly, nostack, raw),
        );
    }

    result
}

pub fn queen_moves(pieces: Bitboard, occ: Bitboard) -> Bitboard {
    let result;
    unsafe {
        std::arch::asm!(
            "call queen_moves",
            in("r8") pieces,
            in("r9") occ,
            out("rax") result,
            out("rdx") _,
            out("xmm0") _,
            out("xmm1") _,
            out("xmm2") _,
            out("xmm3") _,
            out("xmm4") _,
            out("xmm5") _,
            out("xmm6") _,
            options(pure, readonly, nostack, raw),
        );
    }

    result
}

pub fn king_moves(pieces: Bitboard, _occ: Bitboard) -> Bitboard {
    let result;
    unsafe {
        std::arch::asm!(
            "call king_moves",
            in("r8") pieces,
            out("rax") result,
            out("rdx") _,
            options(pure, readonly, nostack, raw),
        );
    }

    result
}

#[allow(improper_ctypes)]
pub fn board_hash(board: &Board) -> u64 {
    let result;
    unsafe {
        std::arch::asm!(
            "call board_hash",
            in ("rsi") board,
            out("rax") result,
            out ("xmm0") _,
            options(pure, readonly, nostack, raw),
        );
    }

    result
}
