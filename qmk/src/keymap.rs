//! This module defines the keymap structure and the `CustomKey` trait for handling custom key behaviors.
//! It provides the foundation for defining and managing keyboard layers and key actions.

use avr_delay::delay_us;
use core::pin;

use keyboard_macros::config_constraints;

use crate::{
    Keyboard, QmkKeyboard,
    usb::events::{add_code, remove_code},
};

/// A trait for defining custom key behaviors.
///
/// This trait allows implementing custom actions for keys when they are pressed or released.
#[config_constraints]
pub trait CustomKey<User: Keyboard>: Send + Sync {
    /// Called when the key is pressed. By default, it delegates to `on_pressed`.
    #[inline(always)]
    fn complete_on_pressed(
        &self,
        keyboard: pin::Pin<&mut QmkKeyboard<User>>,
        _row: u8,
        _column: u8,
    ) {
        self.on_pressed(keyboard);
    }

    /// Defines the action to perform when the key is pressed.
    fn on_pressed(&self, _keyboard: pin::Pin<&mut QmkKeyboard<User>>) {}

    /// Called when the key is released. By default, it delegates to `on_released`.
    #[inline(always)]
    fn complete_on_released(
        &self,
        keyboard: pin::Pin<&mut QmkKeyboard<User>>,
        _row: u8,
        _column: u8,
        _key_actual_layer: u8,
    ) {
        self.on_released(keyboard);
    }

    /// Defines the action to perform when the key is released.
    fn on_released(&self, _keyboard: pin::Pin<&mut QmkKeyboard<User>>) {}
}

/// Represents a basic key with a predefined keycode.
pub struct Key(pub u8);

#[config_constraints]
impl<User: Keyboard> CustomKey<User> for Key {
    /// Adds the keycode to the USB report when the key is pressed.
    fn on_pressed(&self, _keyboard: pin::Pin<&mut QmkKeyboard<User>>) {
        add_code(self.0);
    }

    /// Removes the keycode from the USB report when the key is released.
    fn on_released(&self, _keyboard: pin::Pin<&mut QmkKeyboard<User>>) {
        remove_code(self.0);
    }
}

/// Represents a single layer in the keymap.
///
/// Each layer is a 2D array of custom keys.
pub type Layer<User: Keyboard> =
    [&'static dyn CustomKey<User>; User::MATRIX_ROWS as usize * User::MATRIX_COLUMNS as usize];

/// Represents the entire keymap, consisting of multiple layers.
///
/// The keymap is a 3D array where each layer contains a 2D array of custom keys.
pub type Keymap<User: Keyboard> = [Layer<User>; User::LAYER_COUNT];

#[config_constraints]
impl<User: Keyboard> QmkKeyboard<User> {
    pub fn send_ascii_char_with_delay<const DELAY: u64>(self: pin::Pin<&mut Self>, char: u8) {
        delay_us::<DELAY>();
    }
    pub fn send_ascii_char(self: pin::Pin<&mut Self>, char: u8) {
        self.send_ascii_char_with_delay::<0>(char);
    }
    pub fn send_string_with_delay<T: Iterator<Item = u8>, const DELAY: u64>(mut self: pin::Pin<&mut Self>, string: T) {
        for char in string {
            self.as_mut().send_ascii_char_with_delay::<DELAY>(char);
        }
    }
    pub fn send_string<T: Iterator<Item = u8>>(self: pin::Pin<&mut Self>, string: T) {
        self.send_string_with_delay::<_, 0>(string);
    }
}
