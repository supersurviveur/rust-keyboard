#![no_std]
#![allow(incomplete_features)]
#![feature(abi_avr_interrupt, generic_const_exprs, generic_const_items)]
#![no_main]

use avr_base::pins::{B1, B2, B3, B4, B5, B6, C6, D2, D5, D7, E6, F4, F5, F6, F7, Pin};
use eeprom_magic::eeprom;
use keyboard_macros::progmem;
use keyboard_macros::{entry, image_dimension, include_font_plate};
use qmk::keymap::Keymap;
use qmk::keys::{
    KC_0, KC_1, KC_2, KC_3, KC_4, KC_5, KC_6, KC_7, KC_8, KC_9, KC_A, KC_APOSTROPHE_AND_QUOTE,
    KC_B, KC_BACKSPACE, KC_C, KC_COMMA_AND_LESS_THAN_SIGN, KC_D, KC_DELETE,
    KC_DOT_AND_GREATER_THAN_SIGN, KC_E, KC_ENTER, KC_ESCAPE, KC_F, KC_F20, KC_F21, KC_G,
    KC_GRAVE_ACCENT_AND_TILDE, KC_H, KC_I, KC_J, KC_K, KC_L, KC_LEFT_ALT, KC_LEFT_CONTROL,
    KC_LEFT_GUI, KC_LEFT_SHIFT, KC_M, KC_N, KC_O, KC_P, KC_Q, KC_R, KC_RIGHT_ALT, KC_RIGHT_ARROW,
    KC_RIGHT_CONTROL, KC_RIGHT_GUI, KC_RIGHT_SHIFT, KC_S, KC_SEMICOLON_AND_COLON,
    KC_SLASH_AND_QUESTION_MARK, KC_SPACE, KC_T, KC_TAB, KC_U, KC_V, KC_W, KC_X, KC_Y, KC_Z,
    LayerDown, LayerUp, NO_OP,
};
use qmk::{Keyboard, QmkKeyboard, eeprom, progmem};
// include_image!("images/test.png");

#[eeprom]
static mut TEST: u8 = 42;

type Kb = QmkKeyboard<UserKeyboard>;

#[entry(UserKeyboard)]
fn main(kb: &mut QmkKeyboard<UserKeyboard>) {
    let mut progmemtest = TEST;
    let old_value = progmemtest.read();
    Kb::draw_u8(old_value, 1, 0);
    progmemtest.write(&old_value.wrapping_add(1));
    loop {
        kb.task();
    }
}

struct UserKeyboard {
    a: i8,
}

impl Keyboard for UserKeyboard {
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

    const FONT_DIM: (u8, u8, usize) = image_dimension!("images/fontplate.png");
    const CHAR_WIDTH: u8 = 6;
    const CHAR_HEIGHT: u8 = 13;
    const USER_FONTPLATE: [u8; Self::FONT_SIZE] = include_font_plate!("images/fontplate.png");

    const KEYMAP: progmem::ProgmemRef<Keymap<Self>> = KEYMAP;

    fn rotary_encoder_handler(keyboard: &mut QmkKeyboard<Self>, rotary: i8) {
        keyboard.user.a += rotary;
        QmkKeyboard::<Self>::draw_u8(keyboard.user.a as u8, 0, 100);
    }

    type MatrixRowType = u8;

    fn new() -> Self {
        Self { a: 3 }
    }
}

#[rustfmt::skip]
#[progmem]
static KEYMAP: Keymap<UserKeyboard> = [[
&KC_ESCAPE,&KC_1     ,&KC_2     ,&KC_3     ,&KC_4     ,&KC_5     ,            &KC_6     ,&KC_7     ,&KC_8     ,&KC_9     ,&KC_0     ,&KC_DELETE,
&KC_TAB,   &KC_Q     ,&KC_W     ,&KC_E     ,&KC_R     ,&KC_T     ,            &KC_Y     ,&KC_U     ,&KC_I     ,&KC_O     ,&KC_P     ,&KC_BACKSPACE,
&KC_LEFT_SHIFT,&KC_A ,&KC_S     ,&KC_D     ,&KC_F     ,&KC_G     ,            &KC_H     ,&KC_J     ,&KC_K     ,&KC_L ,&KC_SEMICOLON_AND_COLON,&KC_ENTER,
&KC_LEFT_SHIFT,&KC_Z ,&KC_X     ,&KC_C     ,&KC_V     ,&KC_B     ,            &KC_N     ,&KC_M,&KC_COMMA_AND_LESS_THAN_SIGN,&KC_DOT_AND_GREATER_THAN_SIGN,&KC_SLASH_AND_QUESTION_MARK,&KC_RIGHT_SHIFT,
&KC_LEFT_GUI,&KC_LEFT_ALT,&LayerDown(1),&KC_SPACE,&KC_LEFT_CONTROL,&NO_OP,    &NO_OP,&KC_RIGHT_CONTROL,&KC_SPACE,&KC_RIGHT_ALT,&KC_LEFT_ALT,&KC_RIGHT_GUI,
 ],[
&KC_ESCAPE,&KC_1    ,&KC_2      ,&KC_3     ,&KC_4     ,&KC_5     ,            &KC_6     ,&KC_7     ,&KC_8     ,&KC_9     ,&KC_0     ,&KC_GRAVE_ACCENT_AND_TILDE,
&KC_TAB   ,&KC_W    ,&KC_W      ,&KC_E     ,&KC_R     ,&KC_T     ,            &KC_Y     ,&KC_U     ,&KC_I     ,&KC_O     ,&KC_P     ,&KC_BACKSPACE,
&KC_LEFT_SHIFT,&KC_A,&KC_S      ,&KC_D     ,&KC_F     ,&KC_G     ,            &KC_H     ,&KC_J     ,&KC_K     ,&KC_L,&KC_SEMICOLON_AND_COLON,&KC_APOSTROPHE_AND_QUOTE,
&KC_LEFT_CONTROL,&KC_Z,&KC_X    ,&KC_C     ,&KC_V     ,&KC_B     ,            &KC_F20   ,&KC_F21   ,&KC_N     ,&KC_M,&KC_COMMA_AND_LESS_THAN_SIGN,&KC_DOT_AND_GREATER_THAN_SIGN,
&LayerUp(1),&KC_RIGHT_SHIFT,&KC_LEFT_GUI,&KC_LEFT_ALT,&KC_LEFT_CONTROL,&KC_BACKSPACE,      &KC_SPACE,&KC_ENTER,&NO_OP,&KC_RIGHT_CONTROL,&KC_RIGHT_ALT,&KC_RIGHT_ARROW,
],];
