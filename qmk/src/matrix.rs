use crate::{
    Keyboard, QmkKeyboard,
    atomic::atomic,
    timer::{timer_elapsed, timer_read},
};
use avr_base::pins::{GPIO_INPUT_PIN_DELAY, NO_PIN, Pin};
use avr_delay::{delay_cycles, delay_us};
use keyboard_macros::config_constraints;

pub const MATRIX_IO_DELAY: u64 = 30;

#[config_constraints]
impl<User: Keyboard> QmkKeyboard<User> {
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
        let pin = User::ROW_PINS[row as usize];
        if pin != NO_PIN {
            Self::gpio_atomic_set_pin_output_low(pin);
            return true;
        }
        false
    }
    pub fn unselect_row(row: u8) -> bool {
        let pin = User::ROW_PINS[row as usize];
        if pin != NO_PIN {
            Self::gpio_atomic_set_pin_input_high(pin);
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

    fn matrix_read_cols_on_row(&self, current_matrix: &mut [User::MatrixRowType], current_row: u8) {
        // Start with a clear matrix row
        let mut current_row_value = 0.into();

        if !Self::select_row(current_row) {
            // Select row
            return; // skip NO_PIN row
        }
        delay_cycles::<{ GPIO_INPUT_PIN_DELAY }>();

        // For each col...
        let mut row_shifter = User::MATRIX_ROW_SHIFTER;
        for col_index in 0..User::MATRIX_COLUMNS {
            let pin_state = Self::read_matrix_pin(User::COL_PINS[col_index as usize]);

            // Populate the matrix row with the state of the col pin
            current_row_value |= if pin_state { 0.into() } else { row_shifter };
            row_shifter <<= 1;
        }

        // Unselect row
        Self::unselect_row(current_row);
        delay_us::<{ MATRIX_IO_DELAY }>();

        // Update the matrix
        current_matrix[current_row as usize] = current_row_value;
    }
    pub fn matrix_init(&self) {
        for row in 0..User::ROWS_PER_HAND {
            Self::unselect_row(row);
        }
        for col in 0..User::MATRIX_COLUMNS {
            Self::gpio_atomic_set_pin_input_high(User::COL_PINS[col as usize]);
        }
    }
    pub fn matrix_scan(&mut self) -> bool {
        let mut new_matrix = [0.into(); User::ROWS_PER_HAND as usize];
        for row in 0..User::ROWS_PER_HAND {
            self.matrix_read_cols_on_row(&mut new_matrix, row);
        }

        let changed = if self.raw_matrix == new_matrix {
            false
        } else {
            self.raw_matrix = new_matrix;
            true
        };

        self.debounce(changed) | matrix_post_scan()
    }
    pub fn matrix_task(&mut self) -> bool {
        let changed = self.matrix_scan();
        if changed {
            for row in 0..User::MATRIX_ROWS {
                if self.previous_matrix[row as usize] != self.current_matrix[row as usize] {
                    for column in 0..User::MATRIX_COLUMNS {
                        let current_press =
                            self.current_matrix[row as usize] & (1 << column).into();
                        if self.previous_matrix[row as usize] & (1 << column).into()
                            != current_press
                        {
                            if current_press != 0.into() {
                                self.key_pressed(column, row)
                            } else {
                                self.key_released(column, row)
                            }
                        }
                    }
                }
            }
            self.previous_matrix = self.current_matrix;
        }
        changed
    }
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

#[config_constraints]
impl<User: Keyboard> QmkKeyboard<User> {
    fn debounce(&mut self, changed: bool) -> bool {
        let this_matrix =
            TryInto::<&mut [User::MatrixRowType; User::ROWS_PER_HAND as usize]>::try_into(
                &mut self.current_matrix[User::THIS_HAND_OFFSET as usize
                    ..(User::THIS_HAND_OFFSET + User::ROWS_PER_HAND) as usize],
            )
            .unwrap();

        let mut cooked_changed = false;

        if changed {
            unsafe { DEBOUNCING = true };
            unsafe { DEBOUNCING_TIME = timer_read() };
        } else if unsafe { DEBOUNCING } && unsafe { timer_elapsed(DEBOUNCING_TIME) } >= DEBOUNCE {
            if *this_matrix != self.raw_matrix {
                *this_matrix = self.raw_matrix;
                cooked_changed = true;
            }
            unsafe { DEBOUNCING = false };
        }

        cooked_changed
    }
}
