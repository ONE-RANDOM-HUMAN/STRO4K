#![warn(unsafe_op_in_unsafe_fn)]
#![feature(core_intrinsics)]
#![feature(new_uninit)]

pub mod consts;
pub mod evaluate;
pub mod game;
pub mod movegen;
pub mod moveorder;
pub mod position;
pub mod search;
pub mod tt;

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Only 64-bit is supported");

#[cfg(not(target_arch = "x86_64"))]
compile_error!("Only x86 is supported");

#[cfg(not(target_feature = "aes"))]
compile_error!("aes-ni is required");
