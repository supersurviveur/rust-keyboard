use core::pin;

use crate::{
    Keyboard, QmkKeyboard,
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
    pub fn matrix_init(&self) {
        for row in 0..User::ROWS_PER_HAND {
            Self::unselect_row(row);
        }
        for col in 0..User::MATRIX_COLUMNS {
            Self::gpio_atomic_set_pin_input_high(User::COL_PINS[col as usize]);
        }
    }
    pub fn matrix_scan(mut self: pin::Pin<&mut Self>) -> bool {
        let mut new_matrix = [0.into(); User::ROWS_PER_HAND as usize];
        for row in 0..User::ROWS_PER_HAND {
            self.matrix_read_cols_on_row(&mut new_matrix, row);
        }

        let changed = if self.raw_matrix == new_matrix {
            false
        } else {
            let this = self.as_mut().project();
            *this.raw_matrix = new_matrix;
            true
        };

        self.debounce(changed)
    }
    pub fn matrix_task(mut self: pin::Pin<&mut Self>) -> bool
    where
        User: InterruptsHandler<User>,
    {
        let our_matrix_changed = self.as_mut().matrix_scan();
        self.as_mut().serial_task();
        self.key_task(our_matrix_changed)
    }
    pub fn key_task(mut self: pin::Pin<&mut Self>, our_matrix_changed: bool) -> bool {
        let this = self.as_mut().project();
        let changed = our_matrix_changed
            || unsafe {
                this.previous_matrix[User::OTHER_HAND_OFFSET as usize
                    ..(User::OTHER_HAND_OFFSET + User::ROWS_PER_HAND) as usize]
                    .as_mut_array::<{ User::ROWS_PER_HAND as usize }>()
                    .unwrap_unchecked()
                    != this.current_matrix[User::OTHER_HAND_OFFSET as usize
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
                                self.as_mut().key_pressed(column, row)
                            } else {
                                self.as_mut().key_released(column, row)
                            }
                        }
                    }
                }
            }
            let this = self.project();
            *this.previous_matrix = *this.current_matrix;
        }
        changed
    }
}

static mut DEBOUNCING: bool = false;
static mut DEBOUNCING_TIME: u32 = 0;

pub const DEBOUNCE: u32 = 5;

#[config_constraints]
impl<User: Keyboard> QmkKeyboard<User> {
    fn debounce(self: pin::Pin<&mut Self>, changed: bool) -> bool {
        let this = self.project();
        let this_matrix = unsafe {
            this.current_matrix[User::THIS_HAND_OFFSET as usize
                ..User::THIS_HAND_OFFSET as usize + User::ROWS_PER_HAND as usize]
                .as_mut_array()
                .unwrap_unchecked()
        };

        let mut cooked_changed = false;

        if changed {
            unsafe { DEBOUNCING = true };
            unsafe { DEBOUNCING_TIME = timer_read() };
        } else if unsafe { DEBOUNCING } && unsafe { timer_elapsed(DEBOUNCING_TIME) } >= DEBOUNCE {
            if *this_matrix != *this.raw_matrix {
                *this_matrix = *this.raw_matrix;
                cooked_changed = true;
            }
            unsafe { DEBOUNCING = false };
        }

        cooked_changed
    }
}
