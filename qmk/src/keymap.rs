use keyboard_macros::config_constraints;

use crate::{
    Keyboard, QmkKeyboard,
    usb::events::{add_code, remove_code},
};

#[config_constraints]
pub trait CustomKey<User: Keyboard> {
    #[inline(always)]
    fn complete_on_pressed(&self, keyboard: &mut QmkKeyboard<User>, _row: u8, _column: u8) {
        self.on_pressed(keyboard);
    }
    fn on_pressed(&self, _keyboard: &mut QmkKeyboard<User>) {}
    #[inline(always)]
    fn complete_on_released(&self, keyboard: &mut QmkKeyboard<User>, _row: u8, _column: u8) {
        self.on_released(keyboard);
    }
    fn on_released(&self, _keyboard: &mut QmkKeyboard<User>) {}
}

pub struct Key(pub u8);

#[config_constraints]
impl<User: Keyboard> CustomKey<User> for Key {
    fn on_pressed(&self, _keyboard: &mut QmkKeyboard<User>) {
        add_code(self.0);
    }

    fn on_released(&self, _keyboard: &mut QmkKeyboard<User>) {
        remove_code(self.0);
    }
}

#[config_constraints]
#[derive(Debug, Clone, Copy)]
pub struct Layer<User: Keyboard>
where
    [(); User::MATRIX_ROWS as usize * User::MATRIX_COLUMNS as usize]:,
{
    pub keys:
        [&'static dyn CustomKey<User>; User::MATRIX_ROWS as usize * User::MATRIX_COLUMNS as usize],
}

#[config_constraints]
#[derive(Debug, Clone, Copy)]
pub struct Keymap<User: Keyboard> {
    pub layers: [Layer<User>; User::LAYER_COUNT],
}

#[config_constraints]
unsafe impl<User: Keyboard> Sync for Keymap<User> {}

#[config_constraints]
impl<User: Keyboard> Keymap<User> {
    pub const fn new(layers: [Layer<User>; User::LAYER_COUNT]) -> Self {
        Self { layers }
    }
}

#[config_constraints]
impl<User: Keyboard> Layer<User> {
    pub const fn new(
        keys: [&'static dyn CustomKey<User>;
            User::MATRIX_ROWS as usize * User::MATRIX_COLUMNS as usize],
    ) -> Self {
        Self { keys }
    }
}
