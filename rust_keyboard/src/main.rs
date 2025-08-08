#![no_std]
#![allow(internal_features, unused_imports, dead_code)]
#![feature(
    abi_avr_interrupt,
    sync_unsafe_cell,
    core_intrinsics,
    asm_experimental_arch,
    generic_const_exprs,
    generic_const_items
)]
#![no_main]

use avr_base::pins::{B1, B2, B3, B4, B5, B6, C6, D2, D5, D7, E6, F6, F7, Pin};
use avr_base::register::{USBCON, USBE};
use avr_delay::{delay_ms, delay_us};
use keyboard_constants::{CHAR_HEIGHT, CHAR_WIDTH, pins::RED_LED_PIN};
use keyboard_macros::{keymap, qmk_callback, user_config};
use lufa_rs::{USB_Init, USB_USBTask};
use qmk::keymap::{CustomKey, Key, Keymap, Layer};
use qmk::keys::{
    KC_0_AND_CLOSING_PARENTHESIS, KC_1_AND_EXCLAMATION, KC_2_AND_AT, KC_3_AND_HASHMARK,
    KC_4_AND_DOLLAR, KC_5_AND_PERCENTAGE, KC_6_AND_CARET, KC_7_AND_AMPERSAND, KC_8_AND_ASTERISK,
    KC_9_AND_OPENING_PARENTHESIS, KC_A, KC_APOSTROPHE_AND_QUOTE, KC_B, KC_BACKSPACE, KC_C,
    KC_COMMA_AND_LESS_THAN_SIGN, KC_D, KC_DOT_AND_GREATER_THAN_SIGN, KC_E, KC_ENTER, KC_ESCAPE,
    KC_F, KC_F20, KC_F21, KC_G, KC_GRAVE_ACCENT_AND_TILDE, KC_H, KC_I, KC_J, KC_K, KC_L,
    KC_LEFT_ALT, KC_LEFT_CONTROL, KC_LEFT_GUI, KC_LEFT_SHIFT, KC_M, KC_N, KC_O, KC_P, KC_Q, KC_R,
    KC_RIGHT_ALT, KC_RIGHT_ARROW, KC_RIGHT_CONTROL, KC_RIGHT_SHIFT, KC_S, KC_SEMICOLON_AND_COLON,
    KC_SLASH_AND_QUESTION_MARK, KC_SPACE, KC_T, KC_TAB, KC_U, KC_V, KC_W, KC_X, KC_Y, KC_Z,
    LayerDown, LayerUp, NO_OP,
};
use qmk::usb::events::{add_code, hid_task, remove_code, toggle_code};
use qmk::{Keyboard, QmkKeyboard, is_master};
use qmk::{
    graphics,
    init::disable_watchdog,
    mo,
    serial::{
        ERROR, RES, master_exec_transaction, soft_serial_initiator_init, soft_serial_target_init,
    },
    timer::{cycles_elapsed, cycles_read, timer_init, timer_read},
    to,
};

unsafe extern "C" {
    fn send_char(c: u8);
}

use core::{
    arch::{asm, naked_asm},
    intrinsics::abort,
    panic::{self, PanicInfo},
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    RED_LED_PIN.gpio_set_pin_output();
    RED_LED_PIN.gpio_write_pin_low();
    let _ = qmk::graphics::oled_on();
    // qmk::graphics::clear();
    qmk::graphics::draw_text("PANICKED /!\\", 0, 0);
    let _ = qmk::graphics::render(true);
    // abort();
    loop {
        delay_us::<1000000>();
        RED_LED_PIN.gpio_write_pin_high();
        delay_us::<14000>();
        RED_LED_PIN.gpio_write_pin_low();
    }
}

// include_image!("images/test.png");

fn debug(c: char) {
    qmk::graphics::draw_char(c, 0, 0);
    let _ = qmk::graphics::render(true);
}

static mut ERROR_COUNT: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    let mut kb = QmkKeyboard::new(UserKeyboard { a: 3 });
    kb.init();
    loop {
        kb.task();
        if is_master() {
            // delay_us::<1000>();
            master_exec_transaction(qmk::serial::Transaction::Test);
        } else if !unsafe { ERROR } {
            if unsafe { RES } == qmk::serial::CHAINE {
                qmk::graphics::draw_char('o', 0, 0);
            } else {
                qmk::graphics::draw_char('x', 0, 0);
                unsafe { ERROR_COUNT += 1 };

                qmk::graphics::draw_u8(unsafe { ERROR_COUNT }, 0, CHAR_HEIGHT);
            }
        } else {
            qmk::graphics::draw_char('E', 0, 0);
            unsafe { ERROR_COUNT += 1 };

            qmk::graphics::draw_u8(unsafe { ERROR_COUNT }, 0, CHAR_HEIGHT);
        }
    }
}

/* const CS_LOWER: u16 = mo!(1);
const CS_GO_GAME: u16 = to!(2);
const CS_GO_DEF: u16 = to!(0); */
/* keymap! {
    "sofle/rev1",
    {
        KC_ESC,   KC_1,   KC_2,    KC_3,    KC_4,    KC_5,                        KC_6,     KC_7,    KC_8,    KC_9,    KC_0,  KC_GRV,
        KC_TAB,   KC_Q,   KC_W,    KC_E,    KC_R,    KC_T,                        KC_Y,     KC_U,    KC_I,    KC_O,    KC_P,  KC_BSPC,
        KC_LSFT,  KC_A,   KC_S,    KC_D,    KC_F,    KC_G,                        KC_H,     KC_J,    KC_K,    KC_L, KC_SCLN,  KC_QUOT,
        KC_LCTL,  KC_Z,   KC_X,    KC_C,    KC_V,    KC_B, KC_F20,    KC_F21,     KC_N,     KC_M,    KC_COMM, KC_DOT,KC_SLSH, KC_RSFT,
                         KC_LGUI,KC_LALT,KC_LCTL, CS_LOWER, KC_SPC,    KC_ENT, CS_GO_GAME, KC_RCTL, KC_RALT, KC_RIGHT
    },
    {
        _______,   KC_F1,   KC_F2,   KC_F3,   KC_F4,   KC_F5,                       KC_F6,   KC_F7,   KC_F8,   KC_F9,  KC_F10,  KC_F11,
        KC_GRV,    KC_1,    KC_2,    KC_3,    KC_4,    KC_5,                       KC_6,    KC_7,    KC_8,    KC_9,    KC_0,  KC_F12,
        _______, KC_EXLM,   KC_AT, KC_HASH,  KC_DLR, KC_PERC,                       KC_CIRC, KC_AMPR, KC_ASTR, KC_LPRN, KC_RPRN, KC_PIPE,
        _______,  KC_EQL, KC_MINS, KC_PLUS, KC_LCBR, KC_RCBR, _______,       _______, KC_LBRC, KC_RBRC, KC_SCLN, KC_COLN, KC_BSLS, _______,
                             _______, _______, _______, _______, _______,       _______, _______, _______, _______, _______
    },
    {
        XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX,                     XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX
        XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX,                     XXXXXXX, XXXXXXX,  KC_UP,  XXXXXXX, XXXXXXX, XXXXXXX
        XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX,                     XXXXXXX, KC_LEFT, KC_DOWN, KC_RIGHT,XXXXXXX, XXXXXXX
        XXXXXXX, XXXXXXX,   KC_Z,    KC_X,   KC_C,   XXXXXXX, XXXXXXX,    XXXXXXX,XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX
                         XXXXXXX,XXXXXXX,XXXXXXX, XXXXXXX,  KC_SPC,    KC_ENT , CS_GO_DEF, XXXXXXX, XXXXXXX, XXXXXXX
    },
} */

struct UserKeyboard {
    a: u8,
}

impl Keyboard for UserKeyboard {
    const LAYER_COUNT: usize = 2;
    const MATRIX_ROWS: u8 = 10;
    const MATRIX_COLUMNS: u8 = 6;

    const ROW_PINS: [Pin; Self::ROWS_PER_HAND as usize] = [C6, D7, E6, B4, B5];
    const COL_PINS: [Pin; Self::MATRIX_COLUMNS as usize] = [F6, F7, B1, B3, B2, B6];
    const RED_LED_PIN: Pin = D5;
    const SOFT_SERIAL_PIN: Pin = D2;

    const USER_KEYMAP: &'static Keymap<Self> = &KEYMAP;

    fn test(keyboard: &mut QmkKeyboard<Self>) {
        keyboard.user.a = 3;
    }

    type MatrixRowType = u8;
}

#[unsafe(link_section = ".progmem.data")]
static KEYMAP: Keymap<UserKeyboard> = Keymap::new([
    Layer::new([
        &KC_ESCAPE,
        &KC_1_AND_EXCLAMATION,
        &KC_2_AND_AT,
        &KC_3_AND_HASHMARK,
        &KC_4_AND_DOLLAR,
        &KC_5_AND_PERCENTAGE,
        &KC_6_AND_CARET,
        &KC_7_AND_AMPERSAND,
        &KC_8_AND_ASTERISK,
        &KC_9_AND_OPENING_PARENTHESIS,
        &KC_0_AND_CLOSING_PARENTHESIS,
        &KC_GRAVE_ACCENT_AND_TILDE,
        &KC_TAB,
        &KC_Q,
        &KC_W,
        &KC_E,
        &KC_R,
        &KC_T,
        &KC_Y,
        &KC_U,
        &KC_I,
        &KC_O,
        &KC_P,
        &KC_BACKSPACE,
        &KC_LEFT_SHIFT,
        &KC_A,
        &KC_S,
        &KC_D,
        &KC_F,
        &KC_G,
        &KC_H,
        &KC_J,
        &KC_K,
        &KC_L,
        &KC_SEMICOLON_AND_COLON,
        &KC_APOSTROPHE_AND_QUOTE,
        &KC_LEFT_CONTROL,
        &KC_Z,
        &KC_X,
        &KC_C,
        &KC_V,
        &KC_B,
        &KC_F20,
        &KC_F21,
        &KC_N,
        &KC_M,
        &KC_COMMA_AND_LESS_THAN_SIGN,
        &KC_DOT_AND_GREATER_THAN_SIGN,
        // &KC_SLASH_AND_QUESTION_MARK,
        &LayerDown(1),
        &KC_RIGHT_SHIFT,
        &KC_LEFT_GUI,
        &KC_LEFT_ALT,
        &KC_LEFT_CONTROL,
        &KC_BACKSPACE,
        &KC_SPACE,
        &KC_ENTER,
        &NO_OP,
        &KC_RIGHT_CONTROL,
        &KC_RIGHT_ALT,
        &KC_RIGHT_ARROW,
    ]),
    Layer::new([
        &KC_ESCAPE,
        &KC_1_AND_EXCLAMATION,
        &KC_2_AND_AT,
        &KC_3_AND_HASHMARK,
        &KC_4_AND_DOLLAR,
        &KC_5_AND_PERCENTAGE,
        &KC_6_AND_CARET,
        &KC_7_AND_AMPERSAND,
        &KC_8_AND_ASTERISK,
        &KC_9_AND_OPENING_PARENTHESIS,
        &KC_0_AND_CLOSING_PARENTHESIS,
        &KC_GRAVE_ACCENT_AND_TILDE,
        &KC_TAB,
        &KC_W,
        &KC_W,
        &KC_E,
        &KC_R,
        &KC_T,
        &KC_Y,
        &KC_U,
        &KC_I,
        &KC_O,
        &KC_P,
        &KC_BACKSPACE,
        &KC_LEFT_SHIFT,
        &KC_A,
        &KC_S,
        &KC_D,
        &KC_F,
        &KC_G,
        &KC_H,
        &KC_J,
        &KC_K,
        &KC_L,
        &KC_SEMICOLON_AND_COLON,
        &KC_APOSTROPHE_AND_QUOTE,
        &KC_LEFT_CONTROL,
        &KC_Z,
        &KC_X,
        &KC_C,
        &KC_V,
        &KC_B,
        &KC_F20,
        &KC_F21,
        &KC_N,
        &KC_M,
        &KC_COMMA_AND_LESS_THAN_SIGN,
        &KC_DOT_AND_GREATER_THAN_SIGN,
        // &KC_SLASH_AND_QUESTION_MARK,
        &LayerUp(1),
        &KC_RIGHT_SHIFT,
        &KC_LEFT_GUI,
        &KC_LEFT_ALT,
        &KC_LEFT_CONTROL,
        &KC_BACKSPACE,
        &KC_SPACE,
        &KC_ENTER,
        &NO_OP,
        &KC_RIGHT_CONTROL,
        &KC_RIGHT_ALT,
        &KC_RIGHT_ARROW,
    ]),
]);
