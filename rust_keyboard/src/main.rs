#![no_std]
#![allow(internal_features, unused_imports, dead_code)]
#![feature(
    abi_avr_interrupt,
    sync_unsafe_cell,
    core_intrinsics,
    asm_experimental_arch
)]
#![no_main]

use avr_base::register::{USBCON, USBE};
use avr_delay::delay_us;
use keyboard_constants::{matrix::ROWS_PER_HAND, pins::RED_LED_PIN, CHAR_HEIGHT, CHAR_WIDTH};
use keyboard_macros::{keymap, qmk_callback};
use qmk::{
    graphics,
    matrix::{matrix_init, matrix_read_cols_on_row, matrix_scan, MATRIX},
    mo,
    serial::{
        master_exec_transaction, soft_serial_initiator_init, soft_serial_target_init, ERROR, RES,
    },
    timer::{cycles_elapsed, cycles_read, timer_init, timer_read},
    to,
};
use qmk_sys::is_master;

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
        delay_us::<150000>();
        RED_LED_PIN.gpio_write_pin_low();
    }
}

// include_image!("images/test.png");

static mut LAST: bool = false;

fn debug(c: char) {
    qmk::graphics::draw_char(c, 0, 0);
    let _ = qmk::graphics::render(true);
}

fn init() {
    RED_LED_PIN.gpio_set_pin_output();
    RED_LED_PIN.gpio_write_pin_low();
    let _ = qmk::graphics::init_graphics();
    timer_init();
    if is_master() {
        soft_serial_initiator_init();
    } else {
        soft_serial_target_init();
    }
    matrix_init();

    USBCON.write(USBCON & !USBE);

    unsafe { asm!("sei") };
    //     debug('4');
}

/* static mut KEYBOARD_REPORT: qmk_sys::report_keyboard_t = qmk_sys::report_keyboard_t {
    mods: 0,
    reserved: 0,
    keys: [0, qmk::keys::KC_A as u8, 0, 0, 0, 0],
}; */
static mut ERROR_COUNT: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    init();
    loop {
        //         debug('1');
        matrix_scan();
        //         debug('6');
        for i in 0..6 {
            if (unsafe { MATRIX[0] } & 1 << i) != 0 {
                qmk::graphics::draw_char((b'0' + i) as char, 0, i * 13);
                if i == 1 {
                    // unsafe {
                    //     qmk_sys::add_key_to_report(qmk::keys::KC_A as u8);
                    //     qmk_sys::lufa_driver.send_keyboard.unwrap()(&raw mut KEYBOARD_REPORT);
                    // }
                }
            }
        }
        for i in 0..6 {
            if (unsafe { MATRIX[ROWS_PER_HAND as usize] } & 1 << i) != 0 {
                qmk::graphics::draw_char((b'0' + i) as char, 20, i * 13);
                if i == 1 {
                    // unsafe {
                    //     qmk_sys::add_key_to_report(qmk::keys::KC_A as u8);
                    //     qmk_sys::lufa_driver.send_keyboard.unwrap()(&raw mut KEYBOARD_REPORT);
                    // }
                }
            }
        }
        //         debug('7');
        if is_master() {
            delay_us::<1000>();
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
        //         debug('8');
        let _ = qmk::graphics::render(true);
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
