//! This module provides utilities for executing atomic operations.
//! It ensures that critical sections of code are executed without interruptions.

use core::{arch::asm, mem::offset_of};

use keyboard_macros::config_constraints;

use crate::{Keyboard, OmkKeyboard, OmkMetaHolder, OmkShared};

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

#[inline(always)]
#[config_constraints]
/// # Safety
/// Due to the fact that this spawn a new mutable ref on the UnsafeCells, you should not call this function in itself (in the closure passed to itself)
/// That include calling some library function that may at their discretion use that fonction
pub unsafe fn atomic_access<User: Keyboard,F,R,'a>( keyboard: &'a mut OmkKeyboard<User>, f:F) -> R
where F:FnOnce(&'a mut OmkKeyboard<User>, &'a mut OmkShared<User>) -> R
{
    atomic(|| {
        let whole_kb = (keyboard as *mut OmkKeyboard<User>).wrapping_byte_offset(- (offset_of!(OmkMetaHolder<User>,keyboard) as isize)).cast::<OmkMetaHolder<User>>();
        let interupt_memory: &'a mut _ = unsafe {(*whole_kb).shared.get().as_mut_unchecked()};
        f(keyboard,interupt_memory)
    })
}
