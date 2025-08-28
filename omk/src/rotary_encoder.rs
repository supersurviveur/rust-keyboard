use core::marker::PhantomData;

use keyboard_macros::config_constraints;

use crate::{Keyboard, OmkKeyboard, atomic::atomic_access};

#[config_constraints]
/// Represents a rotary encoder with user-defined constraints.
pub struct RotaryEncoder<User: Keyboard> {
    encoder: RotaryState,
    _phantom: PhantomData<User>,
}

/// Represents the state of the rotary encoder.
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
    /// Initializes a new instance of the `RotaryEncoder` struct.
    pub const fn new() -> Self {
        Self {
            encoder: RotaryState {
                state: 0,
                pulses: 0,
            },
            _phantom: PhantomData,
        }
    }

    /// Initializes the rotary encoder pins as input with pull-up resistors.
    pub fn init() {
        User::ROTARY_ENCODER_PIN1.gpio_set_pin_input_high();
        User::ROTARY_ENCODER_PIN2.gpio_set_pin_input_high();
    }

    /// Processes the rotary encoder task and returns the number of rotations since the last call.
    pub fn task(kb: &mut OmkKeyboard<User>) -> i8 {
        unsafe {
            atomic_access(kb, |_, shared| {
                let res = shared.rotary_encoder.encoder.pulses / User::ROTARY_ENCODER_RESOLUTION;
                shared.rotary_encoder.encoder.pulses %= User::ROTARY_ENCODER_RESOLUTION;
                res
            })
        }
    }
}
