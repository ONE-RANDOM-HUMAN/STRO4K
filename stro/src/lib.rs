#![allow(internal_features)]
#![warn(unsafe_op_in_unsafe_fn)]
#![feature(core_intrinsics)]
#![feature(new_uninit)]

#[cfg(feature = "asm")]
pub mod asm;
pub mod consts;
pub mod evaluate;
pub mod game;
pub mod movegen;
pub mod moveorder;
pub mod position;
pub mod search;
pub mod tt;

#[cfg(feature = "asm")]
/// # Safety
/// This must not be run concurrently with other stro code.
pub unsafe fn init() {
    unsafe { asm::init() }
}

#[cfg(not(feature = "asm"))]
/// # Safety
/// This must not be run concurrently with other stro code.
pub unsafe fn init() {}

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Only 64-bit is supported");

#[cfg(not(target_arch = "x86_64"))]
compile_error!("Only x86 is supported");

#[cfg(not(target_feature = "aes"))]
compile_error!("aes-ni is required");

#[cfg(not(target_feature = "avx2"))]
compile_error!("avx2 is required");

#[cfg(not(target_feature = "fma"))]
compile_error!("fma is required");

#[cfg(all(
    feature = "asm",
    not(all(
        target_arch = "x86_64",
        target_os = "linux",
        target_feature = "avx2",
        target_feature = "bmi2",
    ))
))]
compile_error!("asm requires avx2 and bmi2 on x86_64 linux");
