use crate::search::Search;

unsafe extern "C" {
    pub fn root_search_sysv(search: &mut Search, main_thread: bool, max_depth: i32);
}

/// # Safety
/// No asm functions can be called concurrently
pub(crate) unsafe fn init() {
    // Currently does nothing
}
