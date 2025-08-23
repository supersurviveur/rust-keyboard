use core::{cell::UnsafeCell, marker::PhantomData};

use keyboard_macros::config_constraints;
use pin_project::pin_project;

use crate::{atomic::atomic, Keyboard};

// Maybe less or more, depending on how many times the task is called
// const RESOLUTION: u8 = 2;

#[config_constraints]
#[pin_project]
pub struct RotaryEncoder<User: Keyboard> {
    #[pin]
    encoder: RotaryState,
    _phantom: PhantomData<User>,
}

#[pin_project(!Unpin)]
pub(crate) struct RotaryState {
    state: u8,
    pulses: i8,
}

#[config_constraints]
impl<User: Keyboard> Default for RotaryEncoder<User> {
    fn default() -> Self {
        Self::new()
    }
}

/// # Safety
/// You should only call that in aomic context, i guess ?
/// And, the pointer must point correctly.
#[inline(always)]
#[config_constraints]
pub(crate) unsafe fn fast_encoder_task<User: Keyboard>(encoder: *mut RotaryEncoder<User>) {
    const LUT: [i8; 16] = [0, -1, 1, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, -1, 0];
    let (pad_a, pad_b) = (
        User::ROTARY_ENCODER_PIN1.gpio_read_pin(),
        User::ROTARY_ENCODER_PIN2.gpio_read_pin(),
    );
    let new_state = pad_a as u8 | ((pad_b as u8) << 1);
    unsafe {
        (*encoder).encoder.state = (*encoder).encoder.state << 2 | new_state;
        (*encoder).encoder.pulses += LUT[(*encoder).encoder.state as usize % 16];
    }
}

#[config_constraints]
impl<User: Keyboard> RotaryEncoder<User> {
    pub fn new() -> Self {
        Self {
            encoder: RotaryState {
                state: 0,
                pulses: 0,
            },
            _phantom: PhantomData,
        }
    }
    pub fn init() {
        User::ROTARY_ENCODER_PIN1.gpio_set_pin_input_high();
        User::ROTARY_ENCODER_PIN2.gpio_set_pin_input_high();
    }
    pub fn task(encoder: core::pin::Pin<&mut UnsafeCell<Self>>) -> i8 {
        // self.fast_task();
            atomic(|| {
        unsafe {
            let res = (*encoder).as_mut_unchecked().encoder.pulses;
            (*encoder).as_mut_unchecked().encoder.pulses = 0;
            res

        }
            })
    }
}
