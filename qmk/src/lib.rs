#![no_std]
#![feature(
    slice_as_array,
    asm_experimental_arch,
    sync_unsafe_cell,
    abi_avr_interrupt,
    optimize_attribute
)]
// We are on only one proc, with one thread, so there is no need to worry about static mut ref
#![allow(static_mut_refs)]

pub mod atomic;
pub mod graphics;
pub mod i2c;
pub mod init;
pub mod keys;
pub mod matrix;
pub mod pins;
pub mod primitive;
pub mod serial;
pub mod timer;
pub mod usb;
