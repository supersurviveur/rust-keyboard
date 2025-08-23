//! # lufa-rs
//! 
//! This library provides Rust bindings and macros for interacting with LUFA (Lightweight USB Framework for AVRs).

#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unpredictable_function_pointer_comparisons)]
#![allow(clippy::all)]
#![allow(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::bare_urls)]

pub mod macros;
pub use macros::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
