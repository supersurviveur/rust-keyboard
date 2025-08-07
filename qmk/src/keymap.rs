use keyboard_constants::matrix::{MATRIX_COLS, MATRIX_ROWS};

use crate::usb::events::{add_code, remove_code};

pub trait CustomKey {
    fn on_pressed(&self);
    fn on_released(&self);
}

pub struct Key(pub u8);

impl CustomKey for Key {
    fn on_pressed(&self) {
        add_code(self.0);
    }

    fn on_released(&self) {
        remove_code(self.0);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Layer {
    pub keys: [&'static dyn CustomKey; MATRIX_ROWS as usize * MATRIX_COLS as usize],
}

#[derive(Debug, Clone, Copy)]
pub struct Keymap<const LAYER_COUNT: usize> {
    pub layers: [Layer; LAYER_COUNT],
}
pub trait KeymapTrait {
    fn get_layers(&'static self) -> &'static [Layer];
}

impl<const LAYER_COUNT: usize> KeymapTrait for Keymap<LAYER_COUNT> {
    fn get_layers(&'static self) -> &'static [Layer] {
        &self.layers
    }
}

unsafe impl<const LAYER_COUNT: usize> Sync for Keymap<LAYER_COUNT> {}

impl<const LAYER_COUNT: usize> Keymap<LAYER_COUNT> {
    pub const fn new(layers: [Layer; LAYER_COUNT]) -> Self {
        Self { layers }
    }
}

impl Layer {
    pub const fn new(
        keys: [&'static dyn CustomKey; MATRIX_ROWS as usize * MATRIX_COLS as usize],
    ) -> Self {
        Self { keys }
    }
}
