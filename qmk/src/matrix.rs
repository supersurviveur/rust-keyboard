use crate::{
    atomic::atomic,
    timer::{timer_elapsed, timer_read},
};
use avr_base::pins::{Pin, GPIO_INPUT_PIN_DELAY, NO_PIN};
use avr_delay::{delay_cycles, delay_us};
use keyboard_constants::{
    matrix::{MatrixRowType, MATRIX_COLS, MATRIX_ROWS, MATRIX_ROW_SHIFTER, ROWS_PER_HAND},
    pins::{COL_PINS, ROW_PINS},
};
use qmk_sys::is_right;

pub static mut RAW_MATRIX: [u8; ROWS_PER_HAND as usize] = [0; ROWS_PER_HAND as usize];
pub static mut MATRIX: [u8; MATRIX_ROWS as usize] = [0; MATRIX_ROWS as usize];

const THIS_HAND_OFFSET: u8 = if is_right() { ROWS_PER_HAND } else { 0 };
const OTHER_HAND_OFFSET: u8 = ROWS_PER_HAND - THIS_HAND_OFFSET;

pub fn gpio_atomic_set_pin_output_low(pin: Pin) {
    atomic(|| {
        pin.gpio_set_pin_output();
        pin.gpio_write_pin_low();
    })
}
pub fn gpio_atomic_set_pin_input_high(pin: Pin) {
    atomic(|| {
        pin.gpio_set_pin_input_high();
    })
}
pub fn select_row(row: u8) -> bool {
    let pin = ROW_PINS[row as usize];
    if pin != NO_PIN {
        gpio_atomic_set_pin_output_low(pin);
        return true;
    }
    false
}
pub fn unselect_row(row: u8) -> bool {
    let pin = ROW_PINS[row as usize];
    if pin != NO_PIN {
        gpio_atomic_set_pin_input_high(pin);
        return true;
    }
    false
}

pub fn read_matrix_pin(pin: Pin) -> bool {
    if pin != NO_PIN {
        pin.gpio_read_pin()
    } else {
        true
    }
}

pub const MATRIX_IO_DELAY: u64 = 30;

pub fn matrix_read_cols_on_row(current_matrix: &mut [MatrixRowType], current_row: u8) {
    // Start with a clear matrix row
    let mut current_row_value = 0;

    if !select_row(current_row) {
        // Select row
        return; // skip NO_PIN row
    }
    delay_cycles::<{ GPIO_INPUT_PIN_DELAY }>();

    // For each col...
    let mut row_shifter = MATRIX_ROW_SHIFTER;
    for col_index in 0..MATRIX_COLS {
        let pin_state = read_matrix_pin(COL_PINS[col_index as usize]);

        // Populate the matrix row with the state of the col pin
        current_row_value |= if pin_state { 0 } else { row_shifter };
        row_shifter <<= 1;
    }

    // Unselect row
    unselect_row(current_row);
    delay_us::<{ MATRIX_IO_DELAY }>();

    // Update the matrix
    current_matrix[current_row as usize] = current_row_value;
}

pub fn matrix_scan() -> bool {
    let mut new_matrix = [0; ROWS_PER_HAND as usize];
    for row in 0..ROWS_PER_HAND {
        matrix_read_cols_on_row(&mut new_matrix, row);
    }

    let changed = if unsafe { RAW_MATRIX } == new_matrix {
        false
    } else {
        unsafe { RAW_MATRIX = new_matrix };
        true
    };
    let this_matrix = TryInto::<&mut [MatrixRowType; ROWS_PER_HAND as usize]>::try_into(unsafe {
        &mut MATRIX[THIS_HAND_OFFSET as usize..(THIS_HAND_OFFSET + ROWS_PER_HAND) as usize]
    })
    .unwrap();

    
    debounce(unsafe { &mut RAW_MATRIX }, this_matrix, changed) | matrix_post_scan()
}

static mut LAST_CONNECTED: bool = false;

fn matrix_post_scan() -> bool {
    false
    // let mut changed = false;
    // if is_master() {
    //     let mut slave_matrix = [0; ROWS_PER_HAND as usize];
    //     if unsafe {
    //         qmk_sys::transport_master_if_connected(
    //             MATRIX.as_mut_ptr().wrapping_add(THIS_HAND_OFFSET as usize),
    //             slave_matrix.as_mut_ptr(),
    //         )
    //     } {
    //         let other_matrix =
    //             TryInto::<&mut [MatrixRowType; ROWS_PER_HAND as usize]>::try_into(unsafe {
    //                 &mut MATRIX
    //                     [OTHER_HAND_OFFSET as usize..(OTHER_HAND_OFFSET + ROWS_PER_HAND) as usize]
    //             })
    //             .unwrap();
    //         changed = *other_matrix != slave_matrix;

    //         unsafe { LAST_CONNECTED = true };
    //     } else if unsafe { LAST_CONNECTED } {
    //         // reset other half when disconnected
    //         slave_matrix = [0; ROWS_PER_HAND as usize];
    //         changed = true;

    //         unsafe { LAST_CONNECTED = false };
    //     }

    //     if changed {
    //         let other_matrix =
    //             TryInto::<&mut [MatrixRowType; ROWS_PER_HAND as usize]>::try_into(unsafe {
    //                 &mut MATRIX
    //                     [OTHER_HAND_OFFSET as usize..(OTHER_HAND_OFFSET + ROWS_PER_HAND) as usize]
    //             })
    //             .unwrap();
    //         *other_matrix = slave_matrix;
    //     };
    // } else {
    //     unsafe {
    //         qmk_sys::transport_slave(
    //             MATRIX.as_mut_ptr().wrapping_add(OTHER_HAND_OFFSET as usize),
    //             MATRIX.as_mut_ptr().wrapping_add(THIS_HAND_OFFSET as usize),
    //         )
    //     };
    // }

    // return changed;
}

static mut DEBOUNCING: bool = false;
static mut DEBOUNCING_TIME: u32 = 0;

pub const DEBOUNCE: u32 = 5;

fn debounce(
    raw: &mut [MatrixRowType; ROWS_PER_HAND as usize],
    cooked: &mut [MatrixRowType; ROWS_PER_HAND as usize],
    changed: bool,
) -> bool {
    let mut cooked_changed = false;

    if changed {
        unsafe { DEBOUNCING = true };
        unsafe { DEBOUNCING_TIME = timer_read() };
    } else if unsafe { DEBOUNCING } && unsafe { timer_elapsed(DEBOUNCING_TIME) } >= DEBOUNCE {
        if *cooked != *raw {
            *cooked = *raw;
            cooked_changed = true;
        }
        unsafe { DEBOUNCING = false };
    }

    cooked_changed
}
