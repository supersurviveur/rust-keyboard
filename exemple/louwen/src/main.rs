#![no_std]
#![allow(incomplete_features)]
#![feature(
    abi_avr_interrupt,
    generic_const_exprs,
    generic_const_items,
    const_default,
    const_trait_impl,
    sync_unsafe_cell
)]
#![no_main]

use avr_base::pins::{B1, B2, B3, B4, B5, B6, C6, D2, D5, D7, E6, F4, F5, F6, F7, Pin};
use eeprom_magic::eeprom;
use keyboard_macros::progmem;
use keyboard_macros::{entry, image_dimension, include_font_plate};
use omk::keymap::{CustomKey, Key, Keymap};
use omk::keys::{VOLUME_DOWN, VOLUME_UP};
use omk::progmem::ProgmemRef;
use omk::usb::hid_task;
use omk::{Keyboard, OmkKeyboard, eeprom, progmem};
// include_image!("images/test.png");

type Kb = OmkKeyboard<UserKeyboard>;

#[entry(UserKeyboard)]
fn main(kb: &mut Kb) {
    loop {
        kb.task();
    }
}

struct UserKeyboard {
    // a: i8,
}

#[progmem]
static USER_FONTPLATE: [u8; UserKeyboard::FONT_SIZE] =
    include_font_plate!("../images/fontplate.png");

impl Keyboard for UserKeyboard {
    const HAVE_SCREEN: bool = true;
    const LAYER_COUNT: usize = 2;
    const MATRIX_ROWS: u8 = 10;
    const MATRIX_COLUMNS: u8 = 6;

    const ROW_PINS: [Pin; Self::ROWS_PER_HAND as usize] = [C6, D7, E6, B4, B5];
    const COL_PINS: [Pin; Self::MATRIX_COLUMNS as usize] = [F6, F7, B1, B3, B2, B6];
    const RED_LED_PIN: Pin = D5;
    const SOFT_SERIAL_PIN: Pin = D2;
    const LEFT_ENCODER_PIN1: Pin = F5;
    const LEFT_ENCODER_PIN2: Pin = F4;
    const RIGHT_ENCODER_PIN1: Pin = F4;
    const RIGHT_ENCODER_PIN2: Pin = F5;
    const ROTARY_ENCODER_RESOLUTION: i8 = 1;

    const FONT_DIM: (u8, u8, usize) = image_dimension!("../images/fontplate.png");
    const CHAR_WIDTH: u8 = 6;
    const CHAR_HEIGHT: u8 = 13;

    const USER_FONTPLATE: ProgmemRef<[u8; Self::FONT_SIZE]> = USER_FONTPLATE;

    const KEYMAP: progmem::ProgmemRef<Keymap<Self>> = KEYMAP;

    fn rotary_encoder_handler(keyboard: &mut OmkKeyboard<Self>, rotary: i8) {
        let mut repeat_press = |key: &Key, repeat: u8| {
            for _ in 0..repeat {
                key.on_pressed(keyboard);
                hid_task();
                // delay_ms::<1>();
                key.on_released(keyboard);
                hid_task();
                // delay_ms::<1>();
            }
        };
        if omk::is_left() {
            if rotary > 0 {
                repeat_press(VOLUME_UP, rotary as u8);
            } else {
                repeat_press(VOLUME_DOWN, (-rotary) as u8);
            }
        } else {
            if rotary > 0 {
                repeat_press(VOLUME_UP, rotary as u8);
            } else {
                repeat_press(VOLUME_DOWN, (-rotary) as u8);
            }
        }
    }

    type MatrixRowType = u8;
}

impl const Default for UserKeyboard {
    fn default() -> Self {
        Self {}
    }
}

#[rustfmt::skip]
#[progmem]
static KEYMAP: Keymap<UserKeyboard> =
{ use omk::keys::*;
[[
    ESCAPE, KC_1,   KC_2,   KC_3,   KC_4,   KC_5,   KC_6,   KC_7,   KC_8,   KC_9,   KC_0,   DELETE,
    TAB,    KC_Q,   KC_W,   KC_E,   KC_R,   KC_T,   KC_Y,   KC_U,   KC_I,   KC_O,   KC_P,   BCKSPC,
    L_SHFT, KC_A,   KC_S,   KC_D,   KC_F,   KC_G,   KC_H,   KC_J,   KC_K,   KC_L,   SMICLN, ENTER,
    L_SHFT, KC_Z,   KC_X,   KC_C,   KC_V,   KC_B,   KC_N,   KC_M,   COMMA,  DOT,    SLASH,  L_GUI,
    L_GUI,L_CTRL,&LayerHold(1),SPACE,L_GUI, KC_A,   KC_A,  ENTER,  R_SHFT, R_ALT,  BCKSPC, R_CTRL,
    
]
    ,[
    ESCAPE, KC_1,   KC_2,   KC_3,   KC_4,   KC_5,   KC_6,   KC_7,   KC_8,   KC_9,   KC_0,   RESET,
    TAB,    TAB,    HOME,   ARRO_U, END,    PAGE_UP,KPSLAS, KP_7,   KP_8,   KP_9,   KC_P,   BCKSPC,
    L_SHFT, NUMLCK, ARRO_L, ARRO_D, ARRO_R, PAGE_DW,KP_MIN, KP_4,   KP_5,   KP_6,   KP_0,   ENTER,
    L_SHFT, KC_Z,   VOL_DO, MUTE,   VOL_UP, NO_OP,   KC_N,  KP_1,   KP_2,   KP_3,   SLASH,  L_GUI,
    L_GUI,  L_CTRL, NO_OP,  SPACE,  L_GUI,  KC_A ,   KC_A,  ENTER,R_SHFT,  L_ALT,  DELETE,  R_CTRL,
]]};
