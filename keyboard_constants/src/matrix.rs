pub const MATRIX_ROW_SHIFTER: u8 = 1;
pub const MATRIX_COLS: u8 = 6;
pub const MATRIX_ROWS: u8 = 10;
pub const ROWS_PER_HAND: u8 = MATRIX_ROWS / 2;

pub type MatrixRowType = u8; // Smallest type containing at least MATRIX_ROWS bits
