#![no_std]
#![feature(
    slice_as_array,
    asm_experimental_arch,
    sync_unsafe_cell,
    abi_avr_interrupt,
    optimize_attribute,
    generic_const_exprs,
    generic_const_items,
    associated_type_defaults,
    const_trait_impl,
    const_from
)]
// We are on only one proc, with one thread, so there is no need to worry about static mut ref
#![allow(static_mut_refs)]

use avr_base::pins::Pin;
use core::{
    arch::asm,
    ops::{BitOrAssign, ShlAssign},
};
use num_traits::PrimInt;

use keyboard_constants::pins::RED_LED_PIN;
use keyboard_macros::config_constraints;
use lufa_rs::{USB_Init, USB_USBTask};
use num_traits::Unsigned;
// use num::Unsigned;
use qmk_sys::progmem;

use crate::{
    init::disable_watchdog,
    keymap::{CustomKey, Keymap},
    serial::{soft_serial_initiator_init, soft_serial_target_init},
    timer::timer_init,
    usb::events::hid_task,
};

pub mod atomic;
pub mod graphics;
pub mod helper_traits;
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
    type MatrixRowType: Unsigned + PrimInt + BitOrAssign + ShlAssign<u8> + const From<u8> = u8;

    #[config_constraints(Self)]
    const ROW_PINS: [Pin; Self::ROWS_PER_HAND as usize];
    #[config_constraints(Self)]
    const COL_PINS: [Pin; Self::MATRIX_COLUMNS as usize];
    const RED_LED_PIN: Pin;
    const SOFT_SERIAL_PIN: Pin;

    /// This **MUST** be in progmem !
    #[config_constraints(Self)]
    const USER_KEYMAP: &'static Keymap<Self>;

    #[config_constraints(Self)]
    fn test(keyboard: &mut QmkKeyboard<Self>);

    const ROWS_PER_HAND: u8 = Self::MATRIX_ROWS / 2;
    const THIS_HAND_OFFSET: u8 = if is_right() { Self::ROWS_PER_HAND } else { 0 };
    const OTHER_HAND_OFFSET: u8 = Self::ROWS_PER_HAND - Self::THIS_HAND_OFFSET;
    const MATRIX_ROW_SHIFTER: Self::MatrixRowType = 1.into();
}

#[config_constraints]
pub struct QmkKeyboard<User: Keyboard> {
    pub user: User,

    pub raw_matrix: [User::MatrixRowType; User::ROWS_PER_HAND as usize],
    pub previous_matrix: [User::MatrixRowType; User::MATRIX_ROWS as usize],
    pub current_matrix: [User::MatrixRowType; User::MATRIX_ROWS as usize],

    pub layer: u8,
}

#[config_constraints]
impl<User: Keyboard> QmkKeyboard<User> {
    pub fn new(user: User) -> Self {
        Self {
            user,
            raw_matrix: [0.into(); _],
            previous_matrix: [0.into(); _],
            current_matrix: [0.into(); _],
            layer: 0,
        }
    }
}

#[config_constraints]
impl<User: Keyboard> QmkKeyboard<User> {
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
        self.matrix_init();

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
                &User::USER_KEYMAP.layers[layer as usize].keys[(column
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
