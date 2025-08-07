use crate::{
    Keyboard, KeyboardAuto, QmkKeyboard,
    usb::events::{add_code, remove_code},
};

pub trait CustomKey<User: KeyboardAuto> {
    #[inline(always)]
    fn complete_on_pressed(&self, keyboard: &mut QmkKeyboard<User>, _row: u8, _column: u8) {
        self.on_pressed(keyboard);
    }
    fn on_pressed(&self, _keyboard: &mut QmkKeyboard<User>) {}
    fn on_released(&self, _keyboard: &mut QmkKeyboard<User>) {}
}

pub struct Key(pub u8);

impl<User: KeyboardAuto> CustomKey<User> for Key {
    fn on_pressed(&self, _keyboard: &mut QmkKeyboard<User>) {
        add_code(self.0);
    }

    fn on_released(&self, _keyboard: &mut QmkKeyboard<User>) {
        remove_code(self.0);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Layer<User: KeyboardAuto> {
    pub keys: User::KeysArrayType,
}

#[derive(Debug, Clone, Copy)]
pub struct Keymap<User: KeyboardAuto> {
    pub layers: User::LayersArrayType,
}

unsafe impl<User: KeyboardAuto> Sync for Keymap<User> {}

impl<User: KeyboardAuto> Keymap<User> {
    pub const fn new(layers: User::LayersArrayType) -> Self {
        Self { layers }
    }
}

impl<
    User: KeyboardAuto<
        KeysArrayType = [&'static (dyn CustomKey<User> + 'static);
                            User::MATRIX_ROWS as usize * User::MATRIX_COLUMNS as usize],
    >,
> Layer<User>
{
    pub const fn new(
        keys: [&'static dyn CustomKey<User>;
            User::MATRIX_ROWS as usize * User::MATRIX_COLUMNS as usize],
    ) -> Self {
        Self { keys }
    }
}
