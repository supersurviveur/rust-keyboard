#![allow(incomplete_features)]
#![feature(asm_experimental_arch, min_generic_const_args)]
#![no_std]

pub const F_CPU: u64 = 16_000_000;

pub mod pins;
pub mod register;

#[cfg(target_arch = "avr")]
use core::arch::asm;

use crate::register::{WDCE, WDE, WDTCSR};

pub fn reset_to_bootloader() -> ! {
    unsafe {
        let magic_ptr = 0x0800 as *mut u16;
        *magic_ptr = 0x7777;
        asm!("cli");
        WDTCSR.write(WDCE);
        WDTCSR.write(WDE);

        //Wasting cycles until reset is the intended behavior
        #[allow(clippy::empty_loop)]
        loop {}
    }
}
