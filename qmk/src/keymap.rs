use keyboard_constants::matrix::{MATRIX_COLS, MATRIX_ROWS};

use crate::usb::events::{add_code, remove_code};

pub trait CustomKey {
    fn key_pressed(&self);
    fn key_released(&self);
}

pub struct Key(u8);

impl CustomKey for Key {
    fn key_pressed(&self) {
        add_code(self.0);
    }

    fn key_released(&self) {
        remove_code(self.0);
    }
}

pub struct Layer<'a> {
    keys: [&'a dyn CustomKey; MATRIX_ROWS as usize * MATRIX_COLS as usize],
}

pub struct Keymap<'a, const LAYER_COUNT: usize> {
    layers: [Layer<'a>; LAYER_COUNT],
}

unsafe impl<const LAYER_COUNT: usize> Sync for Keymap<'_, LAYER_COUNT> {}

impl<'a, const LAYER_COUNT: usize> Keymap<'a, LAYER_COUNT> {
    pub const fn new(layers: [Layer<'a>; LAYER_COUNT]) -> Self {
        Self { layers }
    }
}

impl<'a> Layer<'a> {
    pub const fn new(
        keys: [&'a dyn CustomKey; MATRIX_ROWS as usize * MATRIX_COLS as usize],
    ) -> Self {
        Self { keys }
    }
}

static TEST: Keymap<1> = Keymap::new([Layer::new([
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
    &Key(50),
])]);
