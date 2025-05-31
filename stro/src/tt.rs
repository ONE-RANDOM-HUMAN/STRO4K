use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::position::Move;

/// One entry in the tt
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bound {
    None,
    Lower,
    Upper,
    Exact,
}

/// Format:
/// Bits 15-0: packed move
/// Bits 31-16: eval
/// Bits 33-32: bound type
/// Bits 47-34: depth
/// Bits 63-48: upper 16 bits of hash
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct TTData(NonZeroU64);

impl TTData {
    pub fn new(mov: Move, bound: Bound, eval: i32, depth: i32, hash: u64) -> Self {
        // 14 bits - truncating is probably sufficient
        let depth = depth.clamp(0, (1 << 14) - 1);

        TTData(
            NonZeroU64::new(
                u64::from(mov.0.get())
                    | (eval as u16 as u64) << 16
                    | (bound as u64) << 32
                    | (depth as u64) << 34
                    | (hash & 0xFFFF_0000_0000_0000),
            )
            .unwrap(), // all zeroes is not a valid move
        )
    }

    pub fn best_move(self) -> Move {
        Move((self.0.get() as u16).try_into().unwrap())
    }

    pub fn bound(self) -> Bound {
        // SAFETY: All values 0-3 are valid
        unsafe { std::mem::transmute((self.0.get() >> 32) as u8 & 0x3) }
    }

    pub fn eval(self) -> i32 {
        i32::from((self.0.get() >> 16) as i16)
    }

    pub fn depth(self) -> i32 {
        (self.0.get() >> 34) as i32 & ((1 << 14) - 1)
    }
}

static DEFAULT_TT: AtomicU64 = AtomicU64::new(0);

#[no_mangle]
static mut TT_PTR: *const AtomicU64 = &raw const DEFAULT_TT;

static mut TT_LEN: NonZeroU64 = NonZeroU64::new(1).unwrap();

#[no_mangle]
static mut TT_MASK: u64 = 0;

/// # Safety
/// The tt must not be accessed during allocation. The current tt must have been created by alloc.
pub unsafe fn alloc(size_in_bytes: NonZeroU64) {
    unsafe {
        // make sure the old tt is deallocated first
        dealloc();

        let size = size_in_bytes.get() as usize / std::mem::size_of::<u64>();

        TT_PTR = Box::leak(Box::new_zeroed_slice(size).assume_init()).as_mut_ptr();
        TT_LEN = (size as u64).try_into().unwrap();
        TT_MASK = ((size + 1).next_power_of_two() >> 1) as u64 - 1;
    }
}

/// # Safety
/// The tt must not be accessed during deallocation. The current tt must have been created by alloc.
pub unsafe fn dealloc() {
    unsafe {
        if !std::ptr::eq(TT_PTR, &raw const DEFAULT_TT) {
            let slice = std::ptr::slice_from_raw_parts_mut(TT_PTR.cast_mut(), TT_LEN.get() as usize);
            drop(Box::from_raw(slice));
            TT_PTR = &raw const DEFAULT_TT;
        }

        TT_LEN = NonZeroU64::new(1).unwrap();
        TT_MASK = 0;
    }
}

pub fn load(hash: u64) -> Option<TTData> {
    let data = unsafe {
        let index = (hash % TT_LEN) as usize;
        (*TT_PTR.add(index)).load(Ordering::Relaxed)
    };

    NonZeroU64::new(data)
        .filter(|x| x.get() >> 48 == hash >> 48)
        .map(TTData)
}

pub fn store(hash: u64, data: TTData) {
    unsafe {
        let index = (hash % TT_LEN) as usize;
        (*TT_PTR.add(index)).store(data.0.get(), Ordering::Relaxed);
    }
}

pub fn clear() {
    unsafe {
        for i in 0..TT_LEN.get() {
            (*TT_PTR.add(i as usize)).store(0, Ordering::Relaxed);
        }
    }
}
