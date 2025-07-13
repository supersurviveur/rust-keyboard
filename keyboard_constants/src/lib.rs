#![no_std]
pub mod matrix;
pub mod pins;

use keyboard_macros::{image_dimension, include_font_plate};




pub const FONT_DIM: (u8,u8,usize) = image_dimension!("images/fontplate.png");
pub const FONT_WIDTH: u16 = FONT_DIM.0 as u16;
pub const FONT_HEIGHT: u16 = FONT_DIM.1 as u16;
pub const CHAR_WIDTH: u8 = 6;
pub const CHAR_HEIGHT: u8 = 13;
pub const CHAR_PER_ROWS: u8 = (FONT_WIDTH / CHAR_WIDTH as u16) as u8;
pub const FONTPLATE: [u8; FONT_DIM.2] = include_font_plate!("images/fontplate.png");
