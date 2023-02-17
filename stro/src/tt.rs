use std::num::{NonZeroU64, NonZeroUsize};

use crate::position::{Move, MoveFlags, Square};

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
        let depth = depth.clamp(-(1 << 13), (1 << 13) - 1);

        TTData(
            NonZeroU64::new(
                u64::from(mov_pack(mov))
                    | (eval as i16 as u64) << 16
                    | (bound as u64) << 32
                    | (depth as u64 & ((1 << 14) - 1)) << 34
                    | (hash & 0xFFFF_0000_0000_0000),
            )
            .unwrap(), // all zeroes is not a valid move
        )
    }

    pub fn best_move(self) -> Move {
        mov_unpack(self.0.get() as u16)
    }

    pub fn bound(self) -> Bound {
        // SAFETY: All values 0-3 are valid
        unsafe { std::mem::transmute((self.0.get() >> 32) as u8 & 0x3) }
    }

    pub fn eval(self) -> i32 {
        i32::from((self.0.get() >> 16) as i16)
    }

    pub fn depth(self) -> i32 {
        let value = (self.0.get() >> 34) as i32;

        // sign extend with arithmetic right shift
        (value << 18) >> 18
    }
}

const MOVE_FLAGS: [u8; 16] = [
    MoveFlags::NONE.0,
    MoveFlags::DOUBLE_PAWN_PUSH.0,
    MoveFlags::QUEENSIDE_CASTLE.0,
    MoveFlags::KINGSIDE_CASTLE.0,
    MoveFlags::CAPTURE.0,
    MoveFlags::EN_PASSANT.0,
    MoveFlags::NONE.0, // These don't exist
    MoveFlags::NONE.0,
    MoveFlags::PROMO.0,
    MoveFlags::PROMO.0 | 0b0100_0000,
    MoveFlags::PROMO.0 | 0b1000_0000,
    MoveFlags::PROMO.0 | 0b1100_0000,
    MoveFlags::PROMO.0 | MoveFlags::CAPTURE.0,
    MoveFlags::PROMO.0 | MoveFlags::CAPTURE.0 | 0b0100_0000,
    MoveFlags::PROMO.0 | MoveFlags::CAPTURE.0 | 0b1000_0000,
    MoveFlags::PROMO.0 | MoveFlags::CAPTURE.0 | 0b1100_0000,
];

fn mov_pack(mov: Move) -> u16 {
    mov.origin as u16
        | (mov.dest as u16) << 6
        | (MOVE_FLAGS.iter().position(|&x| x == mov.flags.0).unwrap() as u16) << 12
}

fn mov_unpack(mov: u16) -> Move {
    Move {
        origin: Square::from_index((mov & 0x3F) as u8).unwrap(),
        dest: Square::from_index((mov >> 6) as u8 & 0x3F).unwrap(),
        flags: MoveFlags(MOVE_FLAGS[(mov >> 12) as usize]),
    }
}

// Not copy to avoid accidental copies
#[derive(Clone, Debug)]
pub struct TT {
    // size will be hard coded in 4k version
    ptr: *mut u64,
    size: usize,
}

unsafe impl Send for TT {}

impl TT {
    pub fn new(size_in_bytes: NonZeroUsize) -> TT {
        let size = (size_in_bytes.get() / 8).max(1);

        // SAFETY: All zeroes is valid for u64
        unsafe {
            TT {
                ptr: Box::leak(Box::new_zeroed_slice(size).assume_init()).as_mut_ptr(),
                size,
            }
        }
    }

    pub fn resize(&mut self, size_in_bytes: NonZeroUsize) {
        std::mem::replace(
            self,
            TT {
                ptr: std::ptr::null_mut(),
                size: 0,
            },
        )
        .dealloc();
        *self = Self::new(size_in_bytes);
    }

    // Not drop for future smp implementation
    pub fn dealloc(self) {
        unsafe {
            let slice = std::ptr::slice_from_raw_parts_mut(self.ptr, self.size);
            drop(Box::from_raw(slice));
        }
    }

    pub fn load(&self, hash: u64) -> Option<TTData> {
        let index = (hash % self.size as u64) as usize;

        let data = unsafe { std::intrinsics::atomic_load_unordered(self.ptr.add(index)) };

        NonZeroU64::new(data)
            .filter(|x| x.get() >> 48 == hash >> 48)
            .map(TTData)
    }

    pub fn store(&self, hash: u64, data: TTData) {
        let index = (hash % self.size as u64) as usize;

        unsafe {
            std::intrinsics::atomic_store_unordered(self.ptr.add(index), data.0.get());
        }
    }

    pub fn clear(&self) {
        for i in 0..self.size {
            unsafe {
                std::intrinsics::atomic_store_unordered(self.ptr.add(i), 0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mov_pack_unpack() {
        for i in 0..=u16::MAX {
            if i >> 12 != 0 && super::MOVE_FLAGS[(i >> 12) as usize] == 0 {
                continue;
            }

            assert_eq!(i, super::mov_pack(super::mov_unpack(i)));
        }
    }
}
