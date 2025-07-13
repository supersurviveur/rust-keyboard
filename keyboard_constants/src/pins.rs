use avr_base::pins::{Pin, B1, B2, B3, B4, B5, B6, C6, D2, D5, D7, E6, F6, F7};

use crate::matrix::{MATRIX_COLS, ROWS_PER_HAND};

pub static ROW_PINS: [Pin; ROWS_PER_HAND as usize] = [C6, D7, E6, B4, B5];
pub static COL_PINS: [Pin; MATRIX_COLS as usize] = [F6, F7, B1, B3, B2, B6];
pub static RED_LED_PIN: Pin = D5;
pub const SOFT_SERIAL_PIN: Pin = D2;
