use core::mem::offset_of;

use keyboard_macros::config_constraints;

use crate::{
    QmkKeyboard,
    rotary_encoder::{RotaryEncoder, fast_encoder_task},
    serial::shared_memory::{MasterSharedMemory, SlaveSharedMemory},
    timer::timer_increment,
};

#[config_constraints]
pub trait InterruptsHandler<User: crate::Keyboard + InterruptsHandler<User>>:
    crate::Keyboard
{
    const KEYBOARD_PTR: *mut QmkKeyboard<User>;
    const SHARED_MEMORY_SLAVE: *mut SlaveSharedMemory<User> = unsafe {
        Self::KEYBOARD_PTR
            .byte_add(offset_of!(QmkKeyboard<User>, slave_shared_memory))
            .cast()
    };
    const SHARED_MEMORY_MASTER: *mut MasterSharedMemory<User> = unsafe {
        Self::KEYBOARD_PTR
            .byte_add(offset_of!(QmkKeyboard<User>, master_shared_memory))
            .cast()
    };
    const ROTARY_ENCODER: *mut RotaryEncoder<User> = unsafe {
        Self::KEYBOARD_PTR
            .byte_add(offset_of!(QmkKeyboard<User>, rotary_encoder))
            .cast()
    };

    /// # Safety
    /// Should only be called by timer interrupt. #[entry] macro should take care of that
    #[inline(always)]
    unsafe fn timer_interrupt() {
        unsafe {
            timer_increment();
            fast_encoder_task(Self::ROTARY_ENCODER);
        }
    }
    /// # Safety
    /// Should only be called by serial interrupt. #[entry] macro should take care of that
    #[inline(always)]
    unsafe fn serial_interrupt() {
        QmkKeyboard::<User>::serial_interrupt();
    }
}
