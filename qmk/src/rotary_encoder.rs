use core::marker::PhantomData;

use keyboard_macros::config_constraints;

use crate::Keyboard;

const RESOLUTION: u8 = 2;

#[config_constraints]
pub struct RotaryEncoder<User: Keyboard> {
    state: u8,
    pulses: i8,
    _phantom: PhantomData<User>,
}

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

#[config_constraints]
impl<User: Keyboard> RotaryEncoder<User> {
    pub fn new() -> Self {
        Self {
            state: 0,
            pulses: 0,
            _phantom: PhantomData,
        }
    }
    pub fn init(&mut self) {
        User::ROTARY_ENCODER_PIN1.gpio_set_pin_input_high();
        User::ROTARY_ENCODER_PIN2.gpio_set_pin_input_high();
    }
    pub fn task(&mut self) -> Direction {
        const LUT: [i8; 16] = [0, -1, 1, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, -1, 0];
        // use mask to get previous state value
        let mut s = self.state & 0b11;

        let (a_is_low, b_is_low) = (
            !User::ROTARY_ENCODER_PIN1.gpio_read_pin(),
            !User::ROTARY_ENCODER_PIN2.gpio_read_pin(),
        );

        // move in the new state
        if a_is_low {
            s |= 0b0100;
        }
        if b_is_low {
            s |= 0b1000;
        }

        // move new state in
        self.state = s >> 2;

        if (s & 0xC) != (s & 0x3) {
            // Add pulse value from the lookup table
            self.pulses += LUT[s as usize & 0xF];
            // Check if we've reached the resolution threshold
            if self.pulses >= RESOLUTION as i8 {
                self.pulses %= RESOLUTION as i8;
                return Direction::CounterClockwise;
            } else if self.pulses <= -(RESOLUTION as i8) {
                self.pulses %= RESOLUTION as i8;
                return Direction::Clockwise;
            }
        }

        Direction::None
    }
}
