#![no_std]
#![feature(asm_experimental_arch)]
#![allow(warnings)]

use core::debug_assert;

// Auto generated bindings
//include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Progmem bindings
pub mod progmem;

// Side

/// Return true if `IS_KEYBOARD_RIGHT` env variable is set, false otherwise
#[inline(always)]
pub const fn side() -> bool {
    match option_env!("IS_KEYBOARD_RIGHT") {
        Some(x) => !x.is_empty(),
        None => false,
    }
}
#[inline(always)]
pub const fn is_left() -> bool {
    !side()
}
#[inline(always)]
pub const fn is_right() -> bool {
    side()
}
#[inline(always)]
pub const fn is_master() -> bool {
    is_left()
}
