use core::pin;

use keyboard_macros::config_constraints;

use crate::{
    Keyboard, QmkKeyboard,
    usb::events::{add_code, remove_code},
};

#[config_constraints]
pub trait CustomKey<User: Keyboard>: Send + Sync {
    #[inline(always)]
    fn complete_on_pressed(&self, keyboard: pin::Pin<&mut QmkKeyboard<User>>, _row: u8, _column: u8) {
        self.on_pressed(keyboard);
    }
    fn on_pressed(&self, _keyboard: pin::Pin<&mut QmkKeyboard<User>>) {}
    #[inline(always)]
    fn complete_on_released(&self, keyboard: pin::Pin<&mut QmkKeyboard<User>>, _row: u8, _column: u8) {
        self.on_released(keyboard);
    }
    fn on_released(&self, _keyboard: pin::Pin<&mut QmkKeyboard<User>>) {}
}

pub struct Key(pub u8);

#[config_constraints]
impl<User: Keyboard> CustomKey<User> for Key {
    fn on_pressed(&self, _keyboard: pin::Pin<&mut QmkKeyboard<User>>) {
        add_code(self.0);
    }

    fn on_released(&self, _keyboard: pin::Pin<&mut QmkKeyboard<User>>) {
        remove_code(self.0);
    }
}

pub type Layer<User: Keyboard> = [&'static dyn CustomKey<User>;User::MATRIX_ROWS as usize * User::MATRIX_COLUMNS as usize];
pub type Keymap<User: Keyboard> = [Layer<User>;User::LAYER_COUNT];
