#![no_std]
#![allow(incomplete_features)]
#![feature(
    abi_avr_interrupt,
    generic_const_exprs,
    generic_const_items,
    const_default,
    const_trait_impl,
    sync_unsafe_cell,
    stmt_expr_attributes
)]
#![no_main]

use avr_base::pins::{B1, B2, B3, B4, B5, B6, C6, D2, D5, D7, E6, F4, F5, F6, F7, Pin};
use keyboard_macros::progmem;
use keyboard_macros::{entry, image_dimension, include_font_plate};
use omk::keymap::Keymap;
use omk::progmem::ProgmemRef;
use omk::usb::set_vertical_wheel_delta;
use omk::{Keyboard, OmkKeyboard, is_left, progmem};

use crate::private::PRIV_K1;

mod private;

type Kb = OmkKeyboard<UserKeyboard>;

#[entry(UserKeyboard)]
fn main(kb: &mut Kb) {
    loop {
        kb.task();
    }
}

struct UserKeyboard {
    a: i8,
}

#[progmem]
static USER_FONTPLATE: [u8; UserKeyboard::FONT_SIZE] =
    include_font_plate!("../images/fontplate.png");

impl Keyboard for UserKeyboard {
    // Change that if you have no screen
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

    fn rotary_encoder_handler(keyboard: &mut OmkKeyboard<Self>, rotary: (i8, i8)) {
        if is_left() {
            keyboard.user.a += rotary.1;
            // set_vertical_wheel_delta(1);
        } else {
            keyboard.user.a += rotary.1;
        }
        OmkKeyboard::<Self>::draw_u8(keyboard.user.a as u8, 0, 100);
    }

    type MatrixRowType = u8;
}

impl const Default for UserKeyboard {
    fn default() -> Self {
        Self { a: 3 }
    }
}

#[progmem]
static KEYMAP: Keymap<UserKeyboard> = {
    use omk::keys::*;
    #[rustfmt::skip]
    [[
        ESCAPE, KC_1,   KC_2,   KC_3,   KC_4,   KC_5,   KC_6,   KC_7,   KC_8,   KC_9,   KC_0,   DELETE,
        TAB,    KC_Q,   KC_W,   KC_E,   KC_R,   KC_T,   KC_Y,   KC_U,   KC_I,   KC_O,   KC_P,   BCKSPC,
        L_SHFT, KC_A,   KC_S,   KC_D,   KC_F,   KC_G,   KC_H,   KC_J,   KC_K,   KC_L,   SMICLN, ENTER,
        L_SHFT, KC_Z,   KC_X,   KC_C,   KC_V,   KC_B,   KC_N,   KC_M,   COMMA,  DOT,    SLASH,  R_SHFT,
        L_GUI,  L_ALT,  &LayerHold(1), SPACE,  L_CTRL, NO_OP,  NO_OP,  R_CTRL, SPACE,  R_ALT,  L_ALT,  R_GUI,
    ],[
        KC_F12, KC_F1,  KC_F2,  KC_F3,  KC_F4,  KC_F5,  KC_F6,  KC_F7,  KC_F8,  KC_F9,  KC_F10, KC_F11,
        TAB,    TAB,    HOME,   ARRO_U, END,    PAGE_UP,PRIV_K1, KP_7,   KP_8,   KP_9,   KC_P,   BCKSPC,
        L_SHFT, CAPLOK, ARRO_L, ARRO_D, ARRO_R, PAGE_DW,KP_MIN, KP_4,   KP_5,   KP_6,   KP_0,   ENTER,
        L_SHFT, KC_Z,   VOL_DO, MUTE,   VOL_UP, NO_OP,  KC_N,   KC_M,   COMMA,  DOT,    SLASH,  R_SHFT,
        L_GUI,  L_ALT,  NO_OP,  SPACE,  L_CTRL, NO_OP,  NO_OP,  R_CTRL, SPACE,  R_ALT,  L_ALT,  R_GUI,
    ]]
};
