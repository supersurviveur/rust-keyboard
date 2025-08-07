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

use core::arch::asm;

use keyboard_constants::{
    matrix::{MATRIX_COLS, MATRIX_ROWS, MatrixRowType, ROWS_PER_HAND},
    pins::RED_LED_PIN,
};
use lufa_rs::{HID_KEYBOARD_SC_H, USB_Init, USB_USBTask};
use qmk_sys::progmem;

use crate::{
    init::disable_watchdog,
    keymap::{CustomKey, Keymap, KeymapTrait},
    matrix::{THIS_HAND_OFFSET, matrix_init},
    serial::{soft_serial_initiator_init, soft_serial_target_init},
    timer::timer_init,
    usb::events::{add_code, hid_task, remove_code},
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
    // type KeymapType: KeymapTrait;
    const LAYER_COUNT: usize;

    /// This **MUST** be in progmem !
    const KEYMAP: &'static Keymap<{ Self::LAYER_COUNT }> where [(); Self::LAYER_COUNT]:;

    fn test(keyboard: &mut QmkKeyboard<Self>);
}
pub trait Keyboard2: Keyboard {
    type KeymapType: KeymapTrait;
    fn keymap() -> &'static Self::KeymapType;
}
impl<T: Keyboard> Keyboard2 for T
where
    [(); Self::LAYER_COUNT]:,
{
    type KeymapType = Keymap<{ Self::LAYER_COUNT }>;
    fn keymap() -> &'static Self::KeymapType {
        Self::KEYMAP
    }
}

pub struct QmkKeyboard<User: Keyboard> {
    pub user: User,

    pub previous_matrix: [MatrixRowType; MATRIX_ROWS as usize],
    pub current_matrix: [MatrixRowType; MATRIX_ROWS as usize],

    pub layer: u8,
}

impl<User: Keyboard> QmkKeyboard<User> {
    pub fn new(user: User) -> Self {
        Self {
            user,
            previous_matrix: [0; MATRIX_ROWS as usize],
            current_matrix: [0; MATRIX_ROWS as usize],
            layer: 0,
        }
    }
}
impl<User: Keyboard2> QmkKeyboard<User> {
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

    pub fn key_pressed(&mut self, column: u8, row: u8) {
        let key = unsafe {
            progmem::read_value(
                &User::keymap().get_layers()[self.layer as usize].keys[(column
                    + (row % ROWS_PER_HAND) * MATRIX_COLS * 2
                    + if row >= ROWS_PER_HAND { MATRIX_COLS } else { 0 })
                    as usize],
            )
        };
        key.on_pressed();
    }

    pub fn key_released(&mut self, column: u8, row: u8) {
        let key = unsafe {
            progmem::read_value(
                &User::keymap().get_layers()[self.layer as usize].keys[(column
                    + (row % ROWS_PER_HAND) * MATRIX_COLS * 2
                    + if row >= ROWS_PER_HAND { MATRIX_COLS } else { 0 })
                    as usize],
            )
        };
        key.on_released();
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
