use core::cell::SyncUnsafeCell;

use crate::atomic::atomic;
use avr_base::{
    register::{CS00, CS01, CS10, OCIE0A, OCR0A, TCCR0A, TCCR1A, TCCR0B, TCCR1B, TCNT1L, TIMSK0, WGM01},
    F_CPU,
};

pub const TIMER_PRESCALER: u8 = 64;
pub const TIMER_RAW_FREQ: u64 = F_CPU / (TIMER_PRESCALER as u64);
pub const TIMER_RAW_TOP: u8 = (TIMER_RAW_FREQ / 1000) as u8;

static TIMER: SyncUnsafeCell<u32> = SyncUnsafeCell::new(0);

// Currently call the clock of QMK, we will remove that when QMK will become useless
// extern "C" fn rust_timer() {
#[unsafe(no_mangle)]
extern "avr-non-blocking-interrupt" fn __vector_21() {
    unsafe {
        let time = core::ptr::read_volatile(TIMER.get());
        core::ptr::write_volatile(TIMER.get(), time + 1);
    }
}

pub fn timer_init() {
    // 1 ms clock
    TCCR0A.write(WGM01);
    TCCR0B.write(CS00 | CS01); // Prescaler to 64 for a 4us clock

    OCR0A.write(TIMER_RAW_TOP);
    TIMSK0.write(OCIE0A);

    // 1 cycle clock
    TCCR1A.write(0);
    TCCR1B.write(CS10); // Prescaler to 1 for a 1 cycle clock
}

#[inline(always)]
pub fn cycles_read() -> u8 {
    TCNT1L.read()
}

#[inline(always)]
pub fn cycles_elapsed(last: u8) -> u8 {
    cycles_read().wrapping_sub(last)
}

pub fn timer_read() -> u32 {
    atomic(|| unsafe { core::ptr::read_volatile(TIMER.get()) })
}

#[inline(always)]
pub fn timer_elapsed(last: u32) -> u32 {
    timer_read().wrapping_sub(last)
}
#[inline(always)]
pub fn timer_elapsed16(last: u32) -> u16 {
    (timer_elapsed(last) & 0xFFFF) as u16
}
#[inline(always)]
pub fn timer_expired(future: u32) -> bool {
    future < timer_read()
}
