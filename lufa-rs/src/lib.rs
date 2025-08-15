#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unpredictable_function_pointer_comparisons)]
#![allow(clippy::all)]

pub mod macros;
pub use macros::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
