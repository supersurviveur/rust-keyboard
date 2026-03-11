#![feature(asm_experimental_arch, linkage)]
#![no_std]

pub const F_CPU: u64 = 16_000_000;

pub mod pins;
pub mod register;

#[cfg(target_arch = "avr")]
use core::arch::asm;

use crate::register::{WDCE, WDE, WDTCSR};

/// AVR startup routine. The equivalent of libc _start.
/// This code sets up:
/// -cleared register as ABI mandate
/// -stack pointer
/// -SRAM initial state from .data,
/// -cleared .bss,
/// then calls main().
#[unsafe(no_mangle)]
pub extern "C" fn _rust_start() -> ! {
    unsafe {
        // r1 = 0 (clear register, ABI requirement)
        asm!("eor r1, r1");

        // Clear SREG (status register at IO 0x3F)
        asm!("out 0x3F, r1");

        // Setup stack pointer
        core::arch::asm!(
            "ldi r28, lo8(__stack)",
            "ldi r29, hi8(__stack)",
            "out 0x3E, r29", // SPH
            "out 0x3D, r28", // SPL
        );

        unsafe extern "C" {
            unsafe fn __do_clear_bss();
            unsafe fn __do_copy_data();
        }
        __do_copy_data();
        __do_clear_bss();
        panic!();
        // Call main()
        unsafe extern "Rust" {
            unsafe fn main() -> !;
        }
        main();
    }
}

pub fn reset_to_bootloader() -> ! {
    unsafe {
        let magic_ptr = 0x0800 as *mut u16;
        *magic_ptr = 0x7777;
        asm!("cli");
        WDTCSR.write(WDCE);
        WDTCSR.write(WDE);
        loop {}
    }
}
