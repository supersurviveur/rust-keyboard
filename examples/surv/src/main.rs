#![no_std]
#![allow(incomplete_features)]
#![feature(
    abi_avr_interrupt,
    generic_const_exprs,
    generic_const_items,
    const_default,
    const_trait_impl,
    sync_unsafe_cell,
    stmt_expr_attributes,
    min_generic_const_args,
    inherent_associated_types
)]
#![no_main]

use avr_base::pins::{B1, B2, B3, B4, B5, B6, C6, D2, D5, D7, E6, F4, F5, F6, F7, Pin};
use keyboard_macros::progmem;
use keyboard_macros::{entry, image_dimension, include_font_plate};
use omk::keymap::Keymap;
use omk::progmem::ProgmemRef;
use omk::usb::set_vertical_wheel_delta;
use omk::{Keyboard, OmkKeyboard, is_left, progmem};

#[cfg(surv_private)]
mod private;

#[cfg(not(surv_private))]
mod private {
    use omk::keys::DummyKey;

    pub const PRIV_K1: &DummyKey = &DummyKey;
}

use crate::private::PRIV_K1;

type Kb = OmkKeyboard<UserKeyboard>;

#[entry(UserKeyboard)]
fn main(kb: &mut Kb) {
    loop {
        kb.task();
    }
}

struct UserKeyboard {
    rotary_state: i8,
}

#[progmem]
static USER_FONTPLATE: [u8; const { UserKeyboard::FONT_DIM.2 }] =
    include_font_plate!("examples/images/fontplate.png");

impl omk::PrivateConfig for UserKeyboard {
    type const ROWS_PER_HAND: usize = const { Self::MATRIX_ROWS / 2 };
    type const MATRIX_KEYS_COUNT: usize = const { Self::MATRIX_ROWS * Self::MATRIX_COLUMNS };
    type const FONT_SIZE: usize = const { Self::FONT_DIM.2 };
    type const FONT_WIDTH: u8 = const { Self::FONT_DIM.0 };
    type const FONT_HEIGHT: u8 = const { Self::FONT_DIM.1 };
}

impl Keyboard for UserKeyboard {
    // Change that if you have no screen
    const HAVE_SCREEN: bool = true;
    type const LAYER_COUNT: usize = 2;
    type const MATRIX_ROWS: usize = 10;
    type const MATRIX_COLUMNS: usize = 6;

    const ROW_PINS: [Pin; Self::ROWS_PER_HAND] = [C6, D7, E6, B4, B5];
    const COL_PINS: [Pin; Self::MATRIX_COLUMNS] = [F6, F7, B1, B3, B2, B6];
    const RED_LED_PIN: Pin = D5;
    const SOFT_SERIAL_PIN: Pin = D2;
    const LEFT_ENCODER_PIN1: Pin = F5;
    const LEFT_ENCODER_PIN2: Pin = F4;
    const RIGHT_ENCODER_PIN1: Pin = F4;
    const RIGHT_ENCODER_PIN2: Pin = F5;
    const ROTARY_ENCODER_RESOLUTION: i8 = 4;

    const FONT_DIM: (u8, u8, usize) = image_dimension!("examples/images/fontplate.png");
    type const CHAR_WIDTH: u8 = 6;
    type const CHAR_HEIGHT: u8 = 13;

    const USER_FONTPLATE: ProgmemRef<[u8; Self::FONT_SIZE]> = USER_FONTPLATE;

    const KEYMAP: progmem::ProgmemRef<Keymap<Self>> = KEYMAP;

    fn rotary_encoder_handler(keyboard: &mut OmkKeyboard<Self>, rotary: (i8, i8)) {
        if is_left() {
            keyboard.user.rotary_state += rotary.1;

            // scroll faster in navigation layer
            let multiplier = if keyboard.layer == 1 { 2 } else { 1 };
            set_vertical_wheel_delta(-rotary.1 * multiplier);
        } else {
            keyboard.user.rotary_state += rotary.0;
        }
        OmkKeyboard::<Self>::draw_u8(keyboard.user.rotary_state as u8, 0, 100);
    }

    type MatrixRowType = u8;
}

impl const Default for UserKeyboard {
    fn default() -> Self {
        Self { rotary_state: 3 }
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
        TAB,    TAB,    HOME,   ARRO_U, END,    PAGE_UP,PRIV_K1, &MouseLeftClick,   &MouseUp,   &MouseRightClick,   RESET,   BCKSPC,
        L_SHFT, CAPLOK, ARRO_L, ARRO_D, ARRO_R, PAGE_DW,KP_MIN, &MouseLeft,   &MouseDown,   &MouseRight,   KP_0,   ENTER,
        L_SHFT, MEDIA_PLAY,   VOL_DO, MUTE,   VOL_UP, NO_OP,  KC_N,   KC_M,   &MouseWheelClick,  DOT,    SLASH,  R_SHFT,
        L_GUI,  L_ALT,  NO_OP,  SPACE,  L_CTRL, NO_OP,  NO_OP,  R_CTRL, SPACE,  R_ALT,  L_ALT,  R_GUI,
    ]]
};
