use crate::position::Bitboard;
use crate::position::{Board, Move};

use crate::game::Game;

use crate::moveorder::HistoryTable;
use crate::search::Search;

#[allow(improper_ctypes)]
extern "C" {
    pub static mut SHIFTS: [u64; 8];

    pub fn start_sysv() -> !;
    pub fn gen_moves_sysv(board: &Board, moves: *mut Move) -> *mut Move;
    pub fn board_is_area_attacked_sysv(board: &Board, area: Bitboard) -> bool;
    pub fn game_is_repetition_sysv(game: &Game<'_>) -> bool;
    pub fn game_make_move_sysv(game: &Game<'_>, mov: u16) -> bool;
    pub fn clear_tt_sysv();
}

#[repr(C)]
pub struct SearchResult {
    mov: Move,
    score: i32
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

#[allow(improper_ctypes)]
pub fn evaluate(board: &Board) -> i32 {
    let mut result;
    unsafe {
        std::arch::asm!(
            r#"
            call evaluate
            "#,
            inout ("rsi") board => _,
            out("rax") result,
            out("rcx") _,
            out("rdx") _,
            out("rdi") _,
            out("r8") _,
            out("r9") _,
            out("r10") _,
            out("r11") _,
            out("xmm0") _,
            out("xmm1") _,
            out("xmm2") _,
            out("xmm3") _,
            out("xmm4") _,
            out("xmm5") _,
            out("xmm6") _,
            options(pure, readonly, raw),
        );
    }

    result
}

#[allow(improper_ctypes)]
pub fn move_sort_history(moves: &mut [Move], history: &HistoryTable) {
    unsafe {
        std::arch::asm!(
            r#"
            lea r15, [rip + cmp_history]
            call sort_moves
            "#,
            out("rax") _,
            out("rcx") _ ,
            out("rdx") _,
            out("rdi") _,
            in("r8") history,
            out("r9") _,
            out("r10") _,
            in("r11") moves.as_mut_ptr(),
            in("r12") moves.len(),
            out("r13") _,
            out("r15") _,
            options(raw),
        );
    }
}

pub fn move_sort_flags(moves: &mut [Move]) {
    unsafe {
        std::arch::asm!(
            r#"
            lea r15, [rip + cmp_flags]
            call sort_moves
            "#,
            out("rax") _,
            out("rcx") _ ,
            out("rdx") _,
            out("rdi") _,
            out("r8") _,
            out("r9") _,
            out("r10") _,
            in("r11") moves.as_mut_ptr(),
            in("r12") moves.len(),
            out("r13") _,
            out("r15") _,
            options(raw),
        );
    }
}

pub fn move_sort_mvvlva(board: &Board, moves: &mut [Move]) {
    unsafe {
        std::arch::asm!(
            r#"
            lea r15, [rip + cmp_mvvlva]
            call sort_moves
            "#,
            out("rax") _,
            out("rcx") _ ,
            out("rdx") _,
            in("rsi") board,
            out("rdi") _,
            in("r8") board.pieces()[board.side_to_move() as usize].as_ptr(),
            out("r9") _,
            out("r10") _,
            in("r11") moves.as_mut_ptr(),
            in("r12") moves.len(),
            out("r13") _,
            out("r15") _,
            options(raw),
        );
    }
}

pub fn alpha_beta(search: &mut Search, alpha: i32, beta: i32, depth: i32, ply: usize) -> Option<i32> {
    let result: i32;
    unsafe {
        std::arch::asm!(
            r#"
            push rbx
            mov rbx, r8
            call alpha_beta
            pop rbx
            "#,
            out("rax") result,
            inout("rcx") depth => _,
            in("rdx") ply,
            in("rsi") alpha,
            in("rdi") beta,
            inout("r8") search => _,
            options(raw),
        );
    }

    (result != i32::MIN).then_some(result)
}
