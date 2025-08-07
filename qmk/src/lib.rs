#![no_std]
#![feature(
    slice_as_array,
    asm_experimental_arch,
    sync_unsafe_cell,
    abi_avr_interrupt,
    optimize_attribute,
    generic_const_exprs,
    generic_const_items
)]
// We are on only one proc, with one thread, so there is no need to worry about static mut ref
#![allow(static_mut_refs)]

use core::{
    arch::asm,
    ops::{Index, IndexMut, Range},
};

use keyboard_constants::{matrix::MatrixRowType, pins::RED_LED_PIN};
use lufa_rs::{USB_Init, USB_USBTask};
use qmk_sys::progmem;

use crate::{
    init::disable_watchdog,
    keymap::{CustomKey, Keymap, Layer},
    matrix::matrix_init,
    serial::{soft_serial_initiator_init, soft_serial_target_init},
    timer::timer_init,
    usb::events::hid_task,
};

pub mod atomic;
pub mod graphics;
pub mod i2c;
pub mod init;
pub mod keymap;
pub mod keys;
pub mod matrix;
pub mod pins;
pub mod primitive;
pub mod serial;
pub mod timer;
pub mod usb;

pub trait Keyboard: Sized + 'static {
    const LAYER_COUNT: usize;

    const MATRIX_ROWS: u8;
    const MATRIX_COLUMNS: u8;
    const ROWS_PER_HAND: u8 = Self::MATRIX_ROWS / 2;

    /// This **MUST** be in progmem !
    const USER_KEYMAP: &'static Keymap<Self> where Self: KeyboardAuto;

    fn test(keyboard: &mut QmkKeyboard<Self>)
    where
        Self: KeyboardAuto;
}

pub trait KeyboardAuto: Keyboard {
    type KeymapType;
    type LayersArrayType: Index<usize, Output = Layer<Self>>;
    type KeysArrayType: Index<usize, Output = &'static dyn CustomKey<Self>>;
    type MatrixType: IndexMut<usize, Output = MatrixRowType>
        + IndexMut<Range<usize>, Output = [MatrixRowType]>
        + Copy;
    type MatrixSplitType: IndexMut<usize, Output = MatrixRowType>
        + IndexMut<Range<usize>, Output = [MatrixRowType]>
        + Copy
        + Eq
    where
        for<'a> &'a mut <Self as KeyboardAuto>::MatrixSplitType: TryFrom<&'a mut [u8]>;
    const KEYMAP: &'static Self::KeymapType;
    fn get_layers() -> &'static Self::LayersArrayType;
}

impl<T: Keyboard> KeyboardAuto for T
where
    [(); Self::LAYER_COUNT]:,
    [(); Self::MATRIX_ROWS as usize * Self::MATRIX_COLUMNS as usize]:,
    [(); Self::ROWS_PER_HAND as usize]:,
{
    type KeymapType = Keymap<Self>;
    type LayersArrayType = [Layer<Self>; Self::LAYER_COUNT];
    type KeysArrayType =
        [&'static dyn CustomKey<Self>; Self::MATRIX_ROWS as usize * Self::MATRIX_COLUMNS as usize];
    type MatrixType = [MatrixRowType; Self::MATRIX_ROWS as usize];
    type MatrixSplitType = [MatrixRowType; Self::ROWS_PER_HAND as usize];

    const KEYMAP: &'static Keymap<Self> = Self::USER_KEYMAP;

    fn get_layers() -> &'static Self::LayersArrayType {
        &Self::KEYMAP.layers
    }
}

pub struct QmkKeyboard<User: KeyboardAuto> {
    pub user: User,

    pub previous_matrix: User::MatrixType,
    pub current_matrix: User::MatrixType,

    pub layer: u8,
}

impl<User: KeyboardAuto<MatrixType = [MatrixRowType; User::MATRIX_ROWS as usize]>>
    QmkKeyboard<User>
{
    pub fn new(user: User) -> Self {
        Self {
            user,
            previous_matrix: [0; User::MATRIX_ROWS as usize],
            current_matrix: [0; User::MATRIX_ROWS as usize],
            layer: 0,
        }
    }
}
impl<User: KeyboardAuto> QmkKeyboard<User> {
    pub fn init(&self) {
        RED_LED_PIN.gpio_set_pin_output();
        RED_LED_PIN.gpio_write_pin_low();
        disable_watchdog();
        let _ = graphics::init_graphics();
        timer_init();
        if is_master() {
            soft_serial_initiator_init();
        } else {
            soft_serial_target_init();
        }
        matrix_init();

        // Needed for the code to works directly after flash, but seems to crash LUFA, which already resolve the problem on flash
        // USBCON.write(USBCON & !USBE);
        unsafe {
            USB_Init();
        }

        // Enable interrupts
        unsafe { asm!("sei") };
    }

    pub fn task(&mut self) {
        hid_task();
        unsafe {
            USB_USBTask();
        }
        self.matrix_task();
        let _ = graphics::render(true);
    }

    pub fn get_layer_up(&mut self, count: u8) -> u8 {
        self.layer - count
    }
    pub fn get_layer_down(&mut self, count: u8) -> u8 {
        self.layer + count
    }
    pub fn layer_up(&mut self, count: u8) {
        self.layer = self.get_layer_up(count);
    }

    pub fn layer_down(&mut self, count: u8) {
        self.layer = self.get_layer_down(count);
    }
    pub fn get_key(&self, layer: u8, column: u8, row: u8) -> &'static dyn CustomKey<User> {
        unsafe {
            progmem::read_value(
                &User::get_layers()[layer as usize].keys[(column
                    + (row % User::ROWS_PER_HAND) * User::MATRIX_COLUMNS * 2
                    + if row >= User::ROWS_PER_HAND {
                        User::MATRIX_COLUMNS
                    } else {
                        0
                    }) as usize],
            )
        }
    }

    pub fn key_pressed(&mut self, column: u8, row: u8) {
        self.get_key(self.layer, column, row).on_pressed(self);
    }

    pub fn key_released(&mut self, column: u8, row: u8) {
        self.get_key(self.layer, column, row).on_released(self);
    }
}

// Side

/// Return true if `IS_KEYBOARD_RIGHT` env variable is set, false otherwise
#[inline(always)]
pub const fn side() -> bool {
    match option_env!("IS_KEYBOARD_RIGHT") {
        Some(x) => !x.is_empty(),
        None => false,
    }
}
#[inline(always)]
pub const fn is_left() -> bool {
    !side()
}
#[inline(always)]
pub const fn is_right() -> bool {
    side()
}
#[inline(always)]
pub const fn is_master() -> bool {
    is_left()
}
