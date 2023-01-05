#![warn(unsafe_op_in_unsafe_fn)]

pub mod consts;
pub mod evaluate;
pub mod game;
pub mod movegen;
pub mod position;
pub mod search;

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Only 64-bit is supported");
