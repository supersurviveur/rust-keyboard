//! This module provides utilities for executing atomic operations.
//! It ensures that critical sections of code are executed without interruptions.

use core::arch::asm;

/// Executes a closure in an atomic context, disabling and restoring interrupts.
///
/// This function ensures that the provided closure `f` is executed without interruptions by disabling interrupts before execution
/// and restoring the interrupt state afterward.
///
/// # Arguments
/// * `f` - A closure to execute in an atomic context.
///
/// # Returns
/// The result of the closure execution.
///
/// # Example
/// ```rust
/// use crate::atomic::atomic;
///
/// let result = atomic(|| {
///     // Critical section code
///     42
/// });
/// assert_eq!(result, 42);
/// ```
#[inline(always)]
pub fn atomic<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let sreg: u8;
    unsafe {
        asm!("in {0}, 0x3F", out(reg) sreg);
        asm!("cli");
    }

    let result = f();

    unsafe {
        asm!("out 0x3F, {0}", in(reg) sreg);
    }

    result
}
