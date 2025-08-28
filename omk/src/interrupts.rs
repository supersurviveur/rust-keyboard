use keyboard_macros::config_constraints;

use crate::{
    OmkKeyboard, OmkMetaHolder,
    rotary_encoder::{RotaryEncoder, fast_encoder_task},
    serial::shared_memory::{MasterSharedMemory, SlaveSharedMemory},
    timer::timer_increment,
};

#[config_constraints]
pub trait InterruptsHandler<User: crate::Keyboard + InterruptsHandler<User>>:
    crate::Keyboard
{
    const KEYBOARD: &OmkMetaHolder<User>;
    const SHARED_MEMORY_SLAVE: *mut SlaveSharedMemory<User> =
        unsafe { &raw mut (*Self::KEYBOARD.shared.get()).slave_memory };
    const SHARED_MEMORY_MASTER: *mut MasterSharedMemory<User> =
        unsafe { &raw mut (*Self::KEYBOARD.shared.get()).master_memory };
    const ROTARY_ENCODER: *mut RotaryEncoder<User> =
        unsafe { &raw mut (*Self::KEYBOARD.shared.get()).rotary_encoder };

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
        OmkKeyboard::<User>::serial_interrupt();
    }
}
