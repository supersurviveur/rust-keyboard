#![no_std]
#![feature(
    asm_experimental_arch,
    sync_unsafe_cell,
    abi_avr_interrupt,
    generic_const_exprs,
    generic_const_items,
    associated_type_defaults,
    const_trait_impl,
    const_from,
    slice_as_array
)]
#![allow(incomplete_features)]
// We are on only one proc, with one thread, so there is no need to worry about static mut ref
#![allow(static_mut_refs)]

use avr_base::{
    pins::Pin,
    register::{USBCON, USBE},
};
use avr_delay::delay_us;
use core::{
    arch::asm,
    ops::{BitOrAssign, ShlAssign, ShrAssign},
    panic::PanicInfo,
};
use num_traits::PrimInt;

use keyboard_macros::config_constraints;
use lufa_rs::{USB_Init, USB_USBTask};
use num_traits::Unsigned;
use qmk_sys::progmem;

use crate::{
    init::disable_watchdog,
    keymap::{CustomKey, Keymap},
    primitive::{Array2D, BinPackedArray},
    rotary_encoder::RotaryEncoder,
    serial::{
        ERROR_COUNT,
        shared_memory::{MasterSharedMemory, SlaveSharedMemory},
    },
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
pub mod rotary_encoder;
pub mod serial;
pub mod timer;
pub mod usb;

pub trait Keyboard: Sized + 'static {
    const LAYER_COUNT: usize;

    const MATRIX_ROWS: u8;
    const MATRIX_COLUMNS: u8;
    /// Smallest type containing at least MATRIX_ROWS bits
    type MatrixRowType: Unsigned
        + PrimInt
        + BitOrAssign
        + ShlAssign<u8>
        + ShrAssign<u8>
        + Copy
        + const From<u8> = u8;

    #[config_constraints(Self)]
    const ROW_PINS: [Pin; Self::ROWS_PER_HAND as usize];
    #[config_constraints(Self)]
    const COL_PINS: [Pin; Self::MATRIX_COLUMNS as usize];
    const LEFT_ENCODER_PIN1: Pin;
    const LEFT_ENCODER_PIN2: Pin;
    const RIGHT_ENCODER_PIN1: Pin;
    const RIGHT_ENCODER_PIN2: Pin;
    const ROTARY_ENCODER_PIN1: Pin = if is_left() {
        Self::LEFT_ENCODER_PIN1
    } else {
        Self::RIGHT_ENCODER_PIN1
    };
    const ROTARY_ENCODER_PIN2: Pin = if is_left() {
        Self::LEFT_ENCODER_PIN2
    } else {
        Self::RIGHT_ENCODER_PIN2
    };

    const RED_LED_PIN: Pin;
    const SOFT_SERIAL_PIN: Pin;

    const FONT_DIM: (u8, u8, usize);
    const FONT_SIZE: usize = Self::FONT_DIM.2;
    const CHAR_WIDTH: u8;
    const CHAR_HEIGHT: u8;
    #[config_constraints(Self)]
    const USER_FONTPLATE: [u8; Self::FONT_SIZE];
    const FONT_WIDTH: u8 = Self::FONT_DIM.0;
    const FONT_HEIGHT: u8 = Self::FONT_DIM.1;
    const CHAR_PER_ROWS: u8 = Self::FONT_WIDTH / Self::CHAR_WIDTH;

    /// This **MUST** be in progmem !
    #[config_constraints(Self)]
    const KEYMAP: &'static Keymap<Self>;

    #[config_constraints(Self)]
    fn test(keyboard: &mut QmkKeyboard<Self>);

    const ROWS_PER_HAND: u8 = Self::MATRIX_ROWS / 2;
    const THIS_HAND_OFFSET: u8 = if is_right() { Self::ROWS_PER_HAND } else { 0 };
    const OTHER_HAND_OFFSET: u8 = Self::ROWS_PER_HAND - Self::THIS_HAND_OFFSET;
    const MATRIX_ROW_SHIFTER: Self::MatrixRowType = if is_left() {
        1
    } else {
        1 << (Self::MATRIX_COLUMNS - 1)
    }
    .into();

    #[config_constraints(Self)]
    const FONTPLATE: Array2D<
        { Self::FONT_WIDTH },
        { Self::FONT_HEIGHT },
        u16,
        BinPackedArray<{ Self::FONT_SIZE }>,
    > = Array2D::from_existing(BinPackedArray {
        data: Self::USER_FONTPLATE,
    });
}

#[config_constraints]
pub struct QmkKeyboard<User: Keyboard> {
    pub user: User,

    pub raw_matrix: [User::MatrixRowType; User::ROWS_PER_HAND as usize],
    pub previous_matrix: [User::MatrixRowType; User::MATRIX_ROWS as usize],
    pub current_matrix: [User::MatrixRowType; User::MATRIX_ROWS as usize],

    pub master_shared_memory: MasterSharedMemory<User>,
    pub slave_shared_memory: SlaveSharedMemory<User>,

    pub layer: u8,

    pub rotary_encoder: RotaryEncoder<User>,
    pub test: u8,
}

#[config_constraints]
impl<User: Keyboard> QmkKeyboard<User> {
    pub fn new(user: User) -> Self {
        Self {
            user,
            raw_matrix: [0.into(); _],
            previous_matrix: [0.into(); _],
            current_matrix: [0.into(); _],
            master_shared_memory: MasterSharedMemory::new(),
            slave_shared_memory: SlaveSharedMemory::new(),
            layer: 0,
            rotary_encoder: RotaryEncoder::new(),
            test: 0,
        }
    }
}

#[config_constraints]
impl<User: Keyboard> QmkKeyboard<User> {
    #[inline(always)]
    pub fn panic_handler(_info: &PanicInfo) -> ! {
        User::RED_LED_PIN.gpio_set_pin_output();
        User::RED_LED_PIN.gpio_write_pin_low();
        let _ = Self::oled_on();
        // Self::clear();
        Self::draw_text("PANICKED /!\\", 0, 0);
        let _ = Self::render(true);
        loop {
            delay_us::<1000000>();
            User::RED_LED_PIN.gpio_write_pin_high();
            delay_us::<14000>();
            User::RED_LED_PIN.gpio_write_pin_low();
        }
    }

    pub fn init(&mut self) {
        User::RED_LED_PIN.gpio_set_pin_output();
        User::RED_LED_PIN.gpio_write_pin_low();
        disable_watchdog();
        Self::init_graphics().unwrap();
        timer_init();
        self.serial_init();
        self.rotary_encoder.init();
        self.matrix_init();

        if is_master() {
            unsafe {
                USB_Init();
            }
        } else {
            // Needed for the code to works directly after flash, but seems to crash LUFA, which already resolve the problem on flash
            USBCON.write(USBCON & !USBE);
        }

        // Enable interrupts
        unsafe { asm!("sei") };
    }

    pub fn task(&mut self) {
        if is_master() {
            hid_task();
            unsafe {
                USB_USBTask();
            }
        }
        let dir = self.rotary_encoder.task();
        match dir {
            rotary_encoder::Direction::Clockwise => {
                self.test -= 1;
            }
            rotary_encoder::Direction::CounterClockwise => {
                self.test += 1;
            }
            rotary_encoder::Direction::None => {}
        }
        Self::draw_u8(self.test, 0, 100);
        Self::draw_u8(unsafe { ERROR_COUNT }, 0, 50);
        let changed = self.matrix_task();
        Self::render(changed).unwrap();
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
                &User::KEYMAP.layers[layer as usize].keys[(column
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
        self.get_key(self.layer, column, row)
            .complete_on_pressed(self, row, column);
    }

    pub fn key_released(&mut self, column: u8, row: u8) {
        self.get_key(self.layer, column, row)
            .complete_on_released(self, row, column);
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
