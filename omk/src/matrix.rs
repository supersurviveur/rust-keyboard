//! This module provides functionality for managing the keyboard matrix.
//! It includes methods for initializing, scanning, and processing the matrix state.

use crate::{
    Keyboard, OmkKeyboard,
    atomic::atomic,
    interrupts::InterruptsHandler,
    is_left,
    timer::{timer_elapsed, timer_read},
};

use avr_base::pins::{GPIO_INPUT_PIN_DELAY, NO_PIN, Pin};
use avr_delay::{delay_cycles, delay_us};
use keyboard_macros::config_constraints;

pub const MATRIX_IO_DELAY: u64 = 30;

#[config_constraints]
impl<User: Keyboard> OmkKeyboard<User> {
    /// Sets a GPIO pin as output and drives it low atomically.
    ///
    /// This is used to select a row in the keyboard matrix.
    pub fn gpio_atomic_set_pin_output_low(pin: Pin) {
        atomic(|| {
            pin.gpio_set_pin_output();
            pin.gpio_write_pin_low();
        })
    }
    /// Sets a GPIO pin as input with a pull-up resistor atomically.
    ///
    /// This is used to unselect a row in the keyboard matrix.
    ///
    pub fn gpio_atomic_set_pin_input_high(pin: Pin) {
        atomic(|| {
            pin.gpio_set_pin_input_high();
        })
    }
    /// Selects a specific row in the keyboard matrix.
    ///
    /// Returns `true` if the row was successfully selected, or `false` if no row is selected.
    pub fn select_row(row: u8) -> bool {
        let pin = User::ROW_PINS[row as usize];
        if pin != NO_PIN {
            Self::gpio_atomic_set_pin_output_low(pin);
            return true;
        }
        false
    }
    /// Unselects a specific row in the keyboard matrix.
    ///
    /// Returns `true` if the row was successfully unselected, or `false` if no row is selected.
    pub fn unselect_row(row: u8) -> bool {
        let pin = User::ROW_PINS[row as usize];
        if pin != NO_PIN {
            Self::gpio_atomic_set_pin_input_high(pin);
            return true;
        }
        false
    }

    /// Reads the state of a specific column pin in the keyboard matrix.
    ///
    /// Returns `true` if the pin is high, or `false` if the pin is low.
    pub fn read_matrix_pin(pin: Pin) -> bool {
        if pin != NO_PIN {
            pin.gpio_read_pin()
        } else {
            true
        }
    }

    /// Reads the columns of a specific row in the keyboard matrix and updates the matrix state.
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
            if is_left() {
                row_shifter <<= 1;
            } else {
                row_shifter >>= 1;
            }
        }

        // Unselect row
        Self::unselect_row(current_row);
        delay_us::<{ MATRIX_IO_DELAY }>();

        // Update the matrix
        current_matrix[current_row as usize] = current_row_value;
    }

    /// Initializes the keyboard matrix by setting all rows and columns to their default states.
    pub fn matrix_init(&self) {
        for row in 0..User::ROWS_PER_HAND {
            Self::unselect_row(row);
        }
        for col in 0..User::MATRIX_COLUMNS {
            Self::gpio_atomic_set_pin_input_high(User::COL_PINS[col as usize]);
        }
    }

    /// Scans the keyboard matrix for changes and updates its state.
    ///
    /// Returns `true` if the matrix state has changed, or `false` otherwise.
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

        self.debounce(changed)
    }

    /// Handles the matrix task, including scanning and processing key events.
    ///
    /// Returns `true` if any key events were detected, or `false` otherwise.
    pub fn matrix_task(&mut self) -> bool
    where
        User: InterruptsHandler<User>,
    {
        let our_matrix_changed = self.matrix_scan();
        self.serial_task();
        self.key_task(our_matrix_changed)
    }

    /// Processes key events based on the current and previous matrix states.
    pub fn key_task(&mut self, our_matrix_changed: bool) -> bool {
        let changed = our_matrix_changed
            || unsafe {
                self.previous_matrix[User::OTHER_HAND_OFFSET as usize
                    ..(User::OTHER_HAND_OFFSET + User::ROWS_PER_HAND) as usize]
                    .as_mut_array::<{ User::ROWS_PER_HAND as usize }>()
                    .unwrap_unchecked()
                    != self.current_matrix[User::OTHER_HAND_OFFSET as usize
                        ..(User::OTHER_HAND_OFFSET + User::ROWS_PER_HAND) as usize]
                        .as_mut_array()
                        .unwrap_unchecked()
            };
        if changed {
            Self::draw_char('c', 0, 26);
            for row in 0..User::MATRIX_ROWS {
                if self.previous_matrix[row as usize] != self.current_matrix[row as usize] {
                    for column in 0..User::MATRIX_COLUMNS {
                        let current_press =
                            self.current_matrix[row as usize] & (1 << column).into();
                        if self.previous_matrix[row as usize] & (1 << column).into()
                            != current_press
                        {
                            if current_press != 0.into() {
                                Self::draw_u8(column, 0, 0);
                                Self::draw_u8(row, 0, 13);
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

static mut DEBOUNCING: bool = false;
static mut DEBOUNCING_TIME: u32 = 0;

pub const DEBOUNCE: u32 = 5;

#[config_constraints]
impl<User: Keyboard> OmkKeyboard<User> {
    /// Debounces the matrix state to filter out noise and ensure stable key detection.
    fn debounce(&mut self, changed: bool) -> bool {
        let self_matrix = unsafe {
            self.current_matrix[User::THIS_HAND_OFFSET as usize
                ..User::THIS_HAND_OFFSET as usize + User::ROWS_PER_HAND as usize]
                .as_mut_array()
                .unwrap_unchecked()
        };

        let mut cooked_changed = false;

        if changed {
            unsafe { DEBOUNCING = true };
            unsafe { DEBOUNCING_TIME = timer_read() };
        } else if unsafe { DEBOUNCING } && unsafe { timer_elapsed(DEBOUNCING_TIME) } >= DEBOUNCE {
            if *self_matrix != self.raw_matrix {
                *self_matrix = self.raw_matrix;
                cooked_changed = true;
            }
            unsafe { DEBOUNCING = false };
        }

        cooked_changed
    }
}
