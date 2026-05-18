use core::{marker::PhantomData, num::Wrapping};

use keyboard_macros::config_constraints;

use crate::InterruptsHandler;
use crate::{Keyboard, OmkKeyboard, atomic::atomic_access, is_master};

/// Represents a rotary encoder with user-defined constraints.
pub struct RotaryEncoder<User: Keyboard> {
    encoder: RotaryState,
    _phantom: PhantomData<User>,
}

impl<User: Keyboard> Clone for RotaryEncoder<User> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<User: Keyboard> Copy for RotaryEncoder<User> {}

/// Represents the state of the rotary encoder.
#[derive(Clone, Copy)]
pub(crate) struct RotaryState {
    state: u8,
    pulses: Wrapping<i8>,
    prev_other_pulses: Wrapping<i8>,
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
pub(crate) unsafe fn fast_encoder_task<User: Keyboard + InterruptsHandler<User>>(
    encoder: *mut RotaryEncoder<User>,
) {
    const LUT: [i8; 16] = [0, -1, 1, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, -1, 0];
    let (pad_a, pad_b) = (
        User::ROTARY_ENCODER_PIN1.gpio_read_pin(),
        User::ROTARY_ENCODER_PIN2.gpio_read_pin(),
    );
    let new_state = pad_a as u8 | ((pad_b as u8) << 1);
    unsafe {
        (*encoder).encoder.state = (*encoder).encoder.state << 2 | new_state;
        let add = LUT[(*encoder).encoder.state as usize % 16];
        (*encoder).encoder.pulses += add;
        if is_master() {
            (*<User as InterruptsHandler<User>>::SHARED_MEMORY_MASTER)
                .master_rotary_encoder_pulses += add;
        } else {
            (*<User as InterruptsHandler<User>>::SHARED_MEMORY_SLAVE)
                .slave_rotary_encoder_pulses += add;
        }
    }
}

impl<User: Keyboard> RotaryEncoder<User> {
    /// Initializes a new instance of the `RotaryEncoder` struct.
    pub const fn new() -> Self {
        Self {
            encoder: RotaryState {
                state: 0,
                pulses: Wrapping(0),
                prev_other_pulses: Wrapping(0),
            },
            _phantom: PhantomData,
        }
    }

    /// Initializes the rotary encoder pins as input with pull-up resistors.
    pub fn init() {
        User::ROTARY_ENCODER_PIN1.gpio_set_pin_input_high();
        User::ROTARY_ENCODER_PIN2.gpio_set_pin_input_high();
    }
}

#[config_constraints]
impl<User: Keyboard> RotaryEncoder<User> {
    /// Processes the rotary encoder task and returns the number of rotations since the last call.
    pub fn task(kb: &mut OmkKeyboard<User>) -> (i8, i8) {
        unsafe {
            atomic_access(kb, |_, shared| {
                let encoder = &mut shared.rotary_encoder.encoder;
                if is_master() {
                    let this = encoder.pulses.0 / User::ROTARY_ENCODER_RESOLUTION;
                    encoder.pulses %= User::ROTARY_ENCODER_RESOLUTION;

                    let other_new = shared.slave_memory.slave_rotary_encoder_pulses
                        / Wrapping(User::ROTARY_ENCODER_RESOLUTION);
                    let other = (other_new - encoder.prev_other_pulses).0;
                    encoder.prev_other_pulses = other_new;
                    (this, other)
                } else {
                    let this = encoder.pulses.0 / User::ROTARY_ENCODER_RESOLUTION;
                    encoder.pulses %= User::ROTARY_ENCODER_RESOLUTION;
                    let other_new = shared.master_memory.master_rotary_encoder_pulses
                        / Wrapping(User::ROTARY_ENCODER_RESOLUTION);
                    let other = (other_new - encoder.prev_other_pulses).0;
                    encoder.prev_other_pulses = other_new;
                    (other, this)
                }
            })
        }
    }
}
