#![no_std]
#![feature(
    asm_experimental_arch,
    sync_unsafe_cell,
    generic_const_exprs,
    generic_const_items,
    associated_type_defaults,
    const_trait_impl,
    const_convert,
    likely_unlikely,
    const_default,
    const_destruct,
    const_array,
    derive_const,
    min_generic_const_args
)]
#![allow(incomplete_features)]
// We are on only one proc, with one thread, so there is no need to worry about static mut ref
#![allow(static_mut_refs)]

use avr_base::{
    pins::Pin,
    register::{USBCON, USBE},
};
use avr_delay::{delay_ms, delay_us};
use core::{
    arch::asm,
    cell::SyncUnsafeCell,
    marker::PhantomData,
    ops::{BitAnd, BitOrAssign, ShlAssign, ShrAssign},
    panic::PanicInfo,
};
mod limited_storage;
use crate::{
    init::disable_watchdog,
    interrupts::InterruptsHandler,
    keymap::{CustomKey, Keymap},
    limited_storage::LimitedStorage,
    primitive::{Array2D, BinPackedArray, IndexByValue, progmem::ProgmemRef},
    rotary_encoder::RotaryEncoder,
    serial::shared_memory::{MasterSharedMemory, SlaveSharedMemory},
    timer::timer_init,
    usb::{events::hid_task, get_mouse_delta, set_mouse_delta},
};
use keyboard_macros::progmem;
pub use limited_storage::Oom;
use lufa_rs::{USB_Init, USB_USBTask};

pub mod atomic;
pub mod graphics;
pub mod i2c;
pub mod init;
pub mod keymap;
pub mod keys;
pub mod matrix;
pub mod primitive;
pub use primitive::{eeprom, progmem};
pub mod interrupts;
pub mod rotary_encoder;
pub mod serial;
pub mod timer;
pub mod usb;

pub trait Keyboard: Sized + const Default + 'static + PrivateConfig {
    type const LAYER_COUNT: usize;
    const HAVE_SCREEN: bool;

    type const MATRIX_ROWS: usize;
    type const MATRIX_COLUMNS: usize;
    /// Smallest type containing at least MATRIX_ROWS bits
    type MatrixRowType: PartialEq
        + BitAnd<Output = Self::MatrixRowType>
        + BitOrAssign
        + ShlAssign<u8>
        + ShrAssign<u8>
        + Copy
        + const From<u8> = u8;

    const ROW_PINS: [Pin; Self::ROWS_PER_HAND];
    const COL_PINS: [Pin; Self::MATRIX_COLUMNS];
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
    const ROTARY_ENCODER_RESOLUTION: i8 = 1;

    const RED_LED_PIN: Pin;
    const SOFT_SERIAL_PIN: Pin;

    const FONT_DIM: (u8, u8, usize);
    type const CHAR_WIDTH: u8;
    type const CHAR_HEIGHT: u8;
    const USER_FONTPLATE: progmem::ProgmemRef<[u8; Self::FONT_SIZE]>;

    const CHAR_PER_ROWS: u8 = Self::FONT_WIDTH / Self::CHAR_WIDTH;

    /// This **MUST** be in progmem !
    const KEYMAP: progmem::ProgmemRef<Keymap<Self>>;

    const MATRIX_ROW_SHIFTER: Self::MatrixRowType = if is_left() {
        1
    } else {
        1 << (Self::MATRIX_COLUMNS - 1)
    }
    .into();

    const MOUSE_BASE_SPEED: u8 = 1;
    const MOUSE_MAX_SPEED: u8 = 12;
    const MOUSE_ACCELERATION: f32 = 1.;

    const FONTPLATE: Array2D<
        { Self::FONT_WIDTH },
        { Self::FONT_HEIGHT },
        u16,
        ProgmemRef<BinPackedArray<{ Self::FONT_SIZE }>>,
    > = Array2D::<_, _, _, ProgmemRef<_>>::from_existing(unsafe { Self::USER_FONTPLATE.cast() });

    fn rotary_encoder_handler(_keyboard: &mut OmkKeyboard<Self>, _rotation: (i8, i8)) {}

    /// A Holder for all suplementary data that you want accessible from the interrupts handlers.
    /// You need to implement Default on it for initialisation.
    type InterruptAccessibleMemory: const Default = ();
}

pub trait PrivateConfig {
    type const ROWS_PER_HAND: usize;
    const THIS_HAND_OFFSET: usize = if is_right() { Self::ROWS_PER_HAND } else { 0 };
    const OTHER_HAND_OFFSET: usize = Self::ROWS_PER_HAND - Self::THIS_HAND_OFFSET;

    type const MATRIX_KEYS_COUNT: usize;

    type const FONT_SIZE: usize;
    type const FONT_WIDTH: u8;
    type const FONT_HEIGHT: u8;
}

pub type PressHandler<User> =
    fn(key: &dyn CustomKey<User>, row: u8, column: u8, keyborad: &mut OmkKeyboard<User>);
pub type UnPressHandler<User> =
    fn(key: &dyn CustomKey<User>, row: u8, column: u8, layer: u8, keyboard: &mut OmkKeyboard<User>);

pub struct OmkKeyboard<User: Keyboard + PrivateConfig> {
    pub user: User,

    pub raw_matrix: [User::MatrixRowType; User::ROWS_PER_HAND],
    pub previous_matrix: [User::MatrixRowType; User::MATRIX_ROWS],
    pub current_matrix: [User::MatrixRowType; User::MATRIX_ROWS],

    pub layer: u8,
    pub keys_actual_layer: [i8; User::MATRIX_KEYS_COUNT],
    pub mouse_state: OmkMouse<User>,

    next_press_handler_override: Option<(PressHandler<User>, u8)>,
    release_handler_overrides: LimitedStorage<10, (UnPressHandler<User>, u8)>,
}

pub struct OmkMouse<User> {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    buttons_held_duration: u8,
    current_speed: u8,
    _phantom: PhantomData<User>,
}

impl<User: Keyboard> const Default for OmkMouse<User> {
    fn default() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
            buttons_held_duration: 0,
            current_speed: User::MOUSE_BASE_SPEED,
            _phantom: PhantomData,
        }
    }
}

pub struct OmkShared<User: Keyboard> {
    pub master_memory: MasterSharedMemory<User>,
    pub slave_memory: SlaveSharedMemory<User>,
    pub rotary_encoder: RotaryEncoder<User>,
    pub user: User::InterruptAccessibleMemory,
}

pub struct OmkMetaHolder<User: Keyboard> {
    pub keyboard: SyncUnsafeCell<OmkKeyboard<User>>,
    pub shared: SyncUnsafeCell<OmkShared<User>>,
}

unsafe impl<User: Keyboard> Sync for OmkShared<User> {}
impl<User: Keyboard> OmkMetaHolder<User> {
    /// # Safety
    /// Should only be called as part of the keyboard_macro::entry, not manually
    pub const unsafe fn new() -> Self {
        Self {
            keyboard: SyncUnsafeCell::new(OmkKeyboard {
                user: User::default(),
                raw_matrix: [0.into(); _],
                previous_matrix: [0.into(); _],
                current_matrix: [0.into(); _],
                layer: 0,
                keys_actual_layer: [0; _],
                mouse_state: OmkMouse::default(),
                next_press_handler_override: None,
                release_handler_overrides: LimitedStorage::new(),
            }),
            shared: SyncUnsafeCell::new(OmkShared {
                master_memory: MasterSharedMemory::new(),
                slave_memory: SlaveSharedMemory::new(),
                rotary_encoder: RotaryEncoder::new(),
                user: User::InterruptAccessibleMemory::default(),
            }),
        }
    }
}

impl<User: Keyboard> OmkKeyboard<User> {
    #[inline(always)]
    pub fn panic_handler(_info: &PanicInfo) -> ! {
        User::RED_LED_PIN.gpio_set_pin_output();
        User::RED_LED_PIN.gpio_write_pin_low();
        let _ = Self::oled_on();
        Self::clear();
        Self::draw_text("/!\\ PANIC".chars(), 0, 0);
        let _ = Self::render(true);
        loop {
            delay_us::<100000>();
            User::RED_LED_PIN.gpio_write_pin_high();
            delay_us::<40000>();
            User::RED_LED_PIN.gpio_write_pin_low();
        }
    }
    pub fn init(&mut self)
    where
        User: InterruptsHandler<User>,
    {
        avr_base::pins::B0.gpio_set_pin_output();
        avr_base::pins::B0.gpio_write_pin_high();
        User::RED_LED_PIN.gpio_set_pin_output();
        User::RED_LED_PIN.gpio_write_pin_high();
        disable_watchdog();
        timer_init();
        let _ = Self::init_graphics();
        self.serial_init();
        RotaryEncoder::<User>::init();
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
    pub fn task(&mut self)
    where
        User: InterruptsHandler<User>,
    {
        let rotary = RotaryEncoder::<User>::task(self);
        User::rotary_encoder_handler(self, rotary);
        let mut changed = rotary.0 != 0 || rotary.1 != 0;
        changed |= self.matrix_task();
        self.mouse_task();
        let _ = Self::render(changed);

        self.usb_task();
    }

    pub fn usb_task(&mut self)
    where
        User: InterruptsHandler<User>,
    {
        if is_master() {
            hid_task();
            unsafe {
                USB_USBTask();
            }
        }
    }
    pub fn mouse_task(&mut self) {
        if self.mouse_state.up
            || self.mouse_state.down
            || self.mouse_state.left
            || self.mouse_state.right
        {
            let (mut x, mut y) = get_mouse_delta();
            self.mouse_state.buttons_held_duration += 1;

            self.mouse_state.current_speed = User::MOUSE_BASE_SPEED
                + libm::roundf(
                    self.mouse_state.buttons_held_duration as f32 * User::MOUSE_ACCELERATION / 20.,
                ) as u8;

            if self.mouse_state.current_speed > User::MOUSE_MAX_SPEED {
                self.mouse_state.current_speed = User::MOUSE_MAX_SPEED;
            }

            if self.mouse_state.up {
                y -= self.mouse_state.current_speed as i8;
            }
            if self.mouse_state.down {
                y += self.mouse_state.current_speed as i8;
            }
            if self.mouse_state.left {
                x -= self.mouse_state.current_speed as i8;
            }
            if self.mouse_state.right {
                x += self.mouse_state.current_speed as i8;
            }
            set_mouse_delta(x, y);
        } else {
            self.mouse_state.buttons_held_duration = 0;
            self.mouse_state.current_speed = User::MOUSE_BASE_SPEED;
        }
    }

    pub fn get_layer_up(&mut self, count: u8) -> u8 {
        self.layer - count
    }
    pub fn get_layer_down(&mut self, count: u8) -> u8 {
        self.layer + count
    }
    pub fn layer_up(&mut self, count: u8) {
        self.layer += count;
    }

    pub fn layer_down(&mut self, count: u8) {
        self.layer -= count;
    }
    pub fn get_key(&self, layer: u8, column: u8, row: u8) -> &'static dyn CustomKey<User> {
        User::KEYMAP
            .at(layer as usize)
            .at((column
                + (row % User::ROWS_PER_HAND as u8) * User::MATRIX_COLUMNS as u8 * 2
                + if row >= User::ROWS_PER_HAND as u8 {
                    User::MATRIX_COLUMNS as u8
                } else {
                    0
                }) as usize)
            .read()
    }

    pub fn key_pressed(&mut self, column: u8, row: u8) {
        self.keys_actual_layer[(row * User::MATRIX_COLUMNS as u8 + column) as usize] =
            self.layer as i8;

        let key = self.get_key(self.layer, column, row);
        match self.next_press_handler_override.take() {
            None => key.complete_on_pressed(self, row, column),
            Some((fun, i)) => {
                self.keys_actual_layer[(row * User::MATRIX_COLUMNS as u8 + column) as usize] =
                    -(i as i8);
                unsafe {
                    self.release_handler_overrides.access(i).1 = self.layer;
                }
                fun(key, row, column, self);
            }
        }
    }

    pub fn key_released(&mut self, column: u8, row: u8) {
        let key_actual_layer =
            self.keys_actual_layer[(row * User::MATRIX_COLUMNS as u8 + column) as usize];
        if key_actual_layer >= 0 {
            self.get_key(key_actual_layer as u8, column, row)
                .complete_on_released(self, row, column, key_actual_layer as u8);
        } else {
            let (fun, layer) = unsafe {
                self.release_handler_overrides
                    .pop((-key_actual_layer) as u8)
            };
            let key = self.get_key(layer, column, row);
            fun(key, row, column, layer, self)
        }
    }
    /// Try to register a new press / release handler pair, if there is available space for the release.
    /// Erase an eventual previous press handler ovveride.
    pub fn register_custom_handle(
        &mut self,
        press: PressHandler<User>,
        unpress: UnPressHandler<User>,
    ) -> Result<(), Oom> {
        match self.next_press_handler_override {
            Some(_) => {
                //return an error
                Err(Oom())
            }
            None => {
                let i: u8 = self.release_handler_overrides.add((unpress, 0))?;
                self.next_press_handler_override = Some((press, i));
                Ok(())
            }
        }
    }

    pub fn send_key_press(&mut self, key: &dyn CustomKey<User>)
    where
        User: InterruptsHandler<User>,
    {
        key.on_pressed(self);
        self.usb_task();
        delay_ms::<10>();
    }
    pub fn send_key_release(&mut self, key: &dyn CustomKey<User>)
    where
        User: InterruptsHandler<User>,
    {
        key.on_released(self);
        self.usb_task();
        delay_ms::<10>();
    }
    pub fn send_key(&mut self, key: &dyn CustomKey<User>)
    where
        User: InterruptsHandler<User>,
    {
        self.send_key_press(key);
        self.send_key_release(key);
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
