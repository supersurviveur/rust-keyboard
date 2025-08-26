use core::cell::SyncUnsafeCell;

use crate::atomic::atomic;
use avr_base::{
    F_CPU,
    register::{
        CS00, CS01, CS10, OCIE0A, OCR0A, TCCR0A, TCCR0B, TCCR1A, TCCR1B, TCNT1L, TIMSK0, WGM01,
    },
};

pub const TIMER_PRESCALER: u8 = 64;
pub const TIMER_RAW_FREQ: u64 = F_CPU / (TIMER_PRESCALER as u64);
pub const TIMER_RAW_TOP: u8 = (TIMER_RAW_FREQ / 1000) as u8;

static TIMER: SyncUnsafeCell<u32> = SyncUnsafeCell::new(0);

/// # Safety
/// Only call from the timer interrupt, or if you want the clock to go faster ...
/// call from timer interrupt is handled by #[keyboard_macros::entry] macro
#[inline(always)]
pub(crate) unsafe fn timer_increment() {
    unsafe {
        let time = TIMER.get().read_volatile();
        TIMER.get().write_volatile(time + 1);
    }
}

/// Initializes the timer with a 1 ms clock and a 1 cycle clock.
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

/// Reads the current value of the 1 cycle clock.
#[inline(always)]
pub fn cycles_read() -> u8 {
    TCNT1L.read()
}

/// Calculates the number of cycles elapsed since the last recorded value.
#[inline(always)]
pub fn cycles_elapsed(last: u8) -> u8 {
    cycles_read().wrapping_sub(last)
}

/// Reads the current timer value in milliseconds.
pub fn timer_read() -> u32 {
    atomic(|| unsafe { core::ptr::read_volatile(TIMER.get()) })
}

/// Calculates the time elapsed in milliseconds since the last recorded value.
///
/// ```rust
/// let last = timer_read();
/// // ... some operations ...
/// let elapsed = timer_elapsed(last);
/// ```
#[inline(always)]
pub fn timer_elapsed(last: u32) -> u32 {
    timer_read().wrapping_sub(last)
}

/// Calculates the time elapsed in milliseconds since the last recorded value, truncated to 16 bits.
///
/// ```rust
/// let last = timer_read();
/// // ... some operations ...
/// let elapsed16 = timer_elapsed16(last);
/// ```
#[inline(always)]
pub fn timer_elapsed16(last: u32) -> u16 {
    (timer_elapsed(last) & 0xFFFF) as u16
}

/// Checks if a given future time has already passed.
///
/// ```rust
/// let future = timer_read() + 1000; // 1 second in the future
/// if timer_expired(future) {
///     // Time has expired
/// }
/// ```
#[inline(always)]
pub fn timer_expired(future: u32) -> bool {
    future < timer_read()
}
