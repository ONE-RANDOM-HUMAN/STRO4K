use crate::{position::Move, search::Search};

#[allow(improper_ctypes)]
extern "C" {
    pub fn root_search_sysv(search: &mut Search, main_thread: bool, max_depth: i32) -> Move;
}

/// # Safety
/// No asm functions can be called concurrently
pub(crate) unsafe fn init() {
    // Currently does nothing
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
