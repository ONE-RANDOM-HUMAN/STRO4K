use crate::{position::Move, search::Search};

#[allow(improper_ctypes)]
extern "C" {
    pub static mut SHIFTS: [u64; 8];
    pub fn root_search_sysv(search: &mut Search, main_thread: bool) -> Move;
}

/// # Safety
/// No asm functions can be called concurrently
pub(crate) unsafe fn init() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe { SHIFTS = [8, 1, 9, 7, 17, 15, 10, 6] })
}

pub fn alpha_beta(
    search: &mut Search,
    alpha: i32,
    beta: i32,
    depth: i32,
    ply: usize,
) -> Option<i32> {
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
