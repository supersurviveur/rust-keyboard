use core::marker::PhantomData;

use avr_base::pins::{F4, F5};
use keyboard_macros::config_constraints;

use crate::Keyboard;



// Maybe less or more, depending on how many times the task is called
// const RESOLUTION: u8 = 2;

#[config_constraints]
pub struct RotaryEncoder<User: Keyboard> {
    _phantom: PhantomData<User>,
}

pub(crate) struct RotaryState {
    state: u8,
    pulses: i8,
}
pub(crate) static mut ROTARY_ENCODER: RotaryState = RotaryState {
    state: 0,
    pulses: 0,
};

#[config_constraints]
impl<User: Keyboard> Default for RotaryEncoder<User> {
    fn default() -> Self {
        Self::new()
    }
}

pub enum Direction {
    /// A clockwise turn
    Clockwise,
    /// A counterclockwise turn
    CounterClockwise,
    /// No change
    None,
}

/// # Safety
/// You should only call that in aomic context, i guess ?
pub(crate) unsafe fn fast_encoder_task<User: Keyboard>() {
        const LUT: [i8; 16] = [0, -1, 1, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, -1, 0];
        let (pad_a, pad_b) = (
            User::ROTARY_ENCODER_PIN1.gpio_read_pin(),
            User::ROTARY_ENCODER_PIN2.gpio_read_pin(),
        );
        let new_state = pad_a as u8 | ((pad_b as u8) << 1);
        unsafe {
            ROTARY_ENCODER.state = ROTARY_ENCODER.state << 2 | new_state;
            ROTARY_ENCODER.pulses += LUT[ROTARY_ENCODER.state as usize % 16];
        }
    
}

#[config_constraints]
impl<User: Keyboard> RotaryEncoder<User> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
    pub fn init(&mut self) {
        User::ROTARY_ENCODER_PIN1.gpio_set_pin_input_high();
        User::ROTARY_ENCODER_PIN2.gpio_set_pin_input_high();
    }
    pub fn task(&mut self) -> i8 {
        // self.fast_task();
        unsafe {
            let res = ROTARY_ENCODER.pulses;
            ROTARY_ENCODER.pulses = 0;
            res
        }
    }
}
