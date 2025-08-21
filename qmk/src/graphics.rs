use core::{cmp::min, iter::Iterator};

use crate::i2c::I2CError;
use crate::{Keyboard, QmkKeyboard, i2c, progmem};

use keyboard_macros::config_constraints;
const OLED_DISPLAY_HEIGHT: u8 = 32;
const OLED_DISPLAY_WIDTH: u8 = 128;
const OLED_MATRIX_SIZE: usize = (OLED_DISPLAY_HEIGHT as usize) * (OLED_DISPLAY_WIDTH as usize) / 8;
const OLED_DISPLAY_ADDRESS: u8 = 60;

use crate::primitive::{
    Array2D, BinPackedArray, Container2D, IndexByValue, IndexByValueMut, SizedView,
};
use crate::timer::{timer_expired, timer_read};

static mut FRAMEBUF_BINARRAY: Array2D<32, 128, u16, BinPackedArray<{ OLED_MATRIX_SIZE }>> =
    Array2D::<32, 128, _, BinPackedArray<{ OLED_MATRIX_SIZE }>>::new();

fn as_u8_buf(
    buffer: &mut Array2D<32, 128, u16, BinPackedArray<{ OLED_MATRIX_SIZE }>>,
) -> SizedView<4, 128, usize, [u8; OLED_MATRIX_SIZE], &mut [u8; OLED_MATRIX_SIZE]> {
    SizedView::<4, 128, _, [u8; OLED_MATRIX_SIZE], &mut [u8; OLED_MATRIX_SIZE]>::new(
        &mut buffer.backend_mut().data,
    )
}

static mut INITIALIZED: bool = false;
static mut OLED_ACTIVE: bool = false;
static mut DIRTY: u16 = 0;
static mut OLED_TIMEOUT: u32 = 60000;
static mut NEXT_OLED_TIMEOUT: u32 = 0;

// I2C OLED related constants
const I2C_CMD: u8 = 0x00;
const I2C_DATA: u8 = 0x40;
const DISPLAY_OFF: u8 = 0xAE;
const DISPLAY_CLOCK: u8 = 0xD5;
const OLED_DISPLAY_CLOCK: u8 = 0x80;
const MULTIPLEX_RATIO: u8 = 0xA8;
const DISPLAY_START_LINE: u8 = 0x40;
const CHARGE_PUMP: u8 = 0x8D;
#[allow(dead_code)]
const SEGMENT_REMAP_INV: u8 = 0xA1;
const COM_SCAN_DEC: u8 = 0xC8;
#[allow(dead_code)]
const COM_SCAN_ASC: u8 = 0xC0;
const DISPLAY_OFFSET: u8 = 0xD3;
const OLED_COM_PIN_OFFSET: u8 = 0x00;
const COM_PINS: u8 = 0xDA;
const OLED_COM_PINS: u8 = 0x02;
const CONTRAST: u8 = 0x81;
const OLED_BRIGHTNESS: u8 = 255;
const PRE_CHARGE_PERIOD: u8 = 0xD9;
const OLED_PRE_CHARGE_PERIOD: u8 = 0xF1;
const VCOM_DETECT: u8 = 0xDB;
const OLED_VCOM_DETECT: u8 = 0x20;
const DISPLAY_ALL_ON_RESUME: u8 = 0xA4;
const NORMAL_DISPLAY: u8 = 0xA6;
const DEACTIVATE_SCROLL: u8 = 0x2E;
const DISPLAY_ON: u8 = 0xAF;
const ADRESS_MODE_SELECT: u8 = 0x20;
#[allow(dead_code)]
const PAGE_ADDR: u8 = 0x02;
const VERTICAL_ADDR: u8 = 0x01;
#[allow(dead_code)]
const HORIZONTAL_ADDR: u8 = 0x00;
const PAGE_SELECT: u8 = 0xB0;
const ROW_LOW_SELECT: u8 = 0x00;
const ROW_HIGH_SELECT: u8 = 0x10;
const AREA_COLS_SELECT: u8 = 0x21;
const AREA_PAGE_SELECT: u8 = 0x22;

// Rendering chunks
const OLED_I2C_TIMEOUT: u16 = 100;
type OledBlockType = u16; // Type to use for segmenting the oled display for smart rendering, use unsigned types only
const ALL_DIRTY: OledBlockType = !(0 as OledBlockType);
const OLED_BLOCK_COUNT: u8 = size_of::<OledBlockType>() as u8 * 8; // 16 (compile time mathed)
const OLED_BLOCK_ROWS: u8 = (OLED_DISPLAY_WIDTH as u16 / OLED_BLOCK_COUNT as u16) as u8; // 8 (compile time mathed)

#[progmem]
static DISPLAY_SETUP1: [u8; 9] = [
    I2C_CMD,
    DISPLAY_OFF,
    DISPLAY_CLOCK,
    OLED_DISPLAY_CLOCK,
    MULTIPLEX_RATIO,
    OLED_DISPLAY_HEIGHT - 1,
    DISPLAY_START_LINE,
    CHARGE_PUMP,
    0x14,
];

#[progmem]
static DISPLAY_SETUP2: [u8; 13] = [
    I2C_CMD,
    COM_PINS,
    OLED_COM_PINS,
    CONTRAST,
    OLED_BRIGHTNESS,
    PRE_CHARGE_PERIOD,
    OLED_PRE_CHARGE_PERIOD,
    VCOM_DETECT,
    OLED_VCOM_DETECT,
    DISPLAY_ALL_ON_RESUME,
    NORMAL_DISPLAY,
    DEACTIVATE_SCROLL,
    DISPLAY_ON,
];

#[progmem]
static DISPLAY_NORMAL: [u8; 12] = [
    I2C_CMD,
    COM_SCAN_DEC,
    DISPLAY_OFFSET,
    OLED_COM_PIN_OFFSET,
    ADRESS_MODE_SELECT,
    VERTICAL_ADDR,
    AREA_COLS_SELECT,
    0,
    127,
    AREA_PAGE_SELECT,
    0,
    3,
];
#[progmem]
static DISPLAY_ON_DATA: [u8; 2] = [I2C_CMD, DISPLAY_ON];
#[progmem]
static DISPLAY_OFF_DATA: [u8; 2] = [I2C_CMD, DISPLAY_OFF];

#[config_constraints]
impl<User: Keyboard> QmkKeyboard<User> {
    #[inline(always)]
    pub fn oled_send_iter<T: Iterator<Item = u8>>(data: T) -> Result<(), I2CError> {
        i2c::i2c_transmit(OLED_DISPLAY_ADDRESS << 1, data, OLED_I2C_TIMEOUT)
    }

    pub fn init_graphics() -> Result<(), I2CError> {
        i2c::i2c_init();
        Self::oled_send_iter(DISPLAY_SETUP1.iter_u8())?;
        Self::oled_send_iter(DISPLAY_NORMAL.iter_u8())?;
        Self::oled_send_iter(DISPLAY_SETUP2.iter_u8())?;
        unsafe { INITIALIZED = true };
        unsafe { OLED_ACTIVE = true };
        unsafe { NEXT_OLED_TIMEOUT = timer_read() + OLED_TIMEOUT };
        Self::clear();
        Ok(())
    }

    pub fn oled_on() -> Result<(), I2CError> {
        if !unsafe { INITIALIZED } {
            Self::init_graphics()?;
            unsafe { OLED_ACTIVE = false };
        }
        if !unsafe { OLED_ACTIVE } {
            Self::oled_send_iter(DISPLAY_ON_DATA.iter_u8())?;
            unsafe { OLED_ACTIVE = true };
            unsafe { NEXT_OLED_TIMEOUT = timer_read() + OLED_TIMEOUT };
        }
        Ok(())
    }
    pub fn oled_off() -> Result<(), I2CError> {
        if !unsafe { INITIALIZED } {
            return Ok(());
        }
        if unsafe { OLED_ACTIVE } {
            Self::oled_send_iter(DISPLAY_OFF_DATA.iter_u8())?;
            unsafe { OLED_ACTIVE = false };
        }
        Ok(())
    }

    //row: 1 to 128
    //col: 1 to 4
    fn oled_goto(row: u8, col_x8: u8) -> Result<(), I2CError> {
        let commands = [
            I2C_CMD,
            PAGE_SELECT | col_x8,
            ROW_LOW_SELECT | (row & 0xF),
            ROW_HIGH_SELECT | (row >> 4),
        ];
        Self::oled_send_iter(commands.into_iter())?;
        Ok(())
    }

    fn oled_draw_lines<
        Backend: IndexByValueMut<usize, Data = u8>,
        T: Container2D<usize, Backend, SliceContainer: IndexByValue<usize, Data = u8>>,
    >(
        starting_row: u8,
        data: T,
    ) -> Result<(), I2CError> {
        Self::oled_goto(starting_row, 0)?;

        Self::oled_send_iter(
            [I2C_DATA].into_iter().chain(
                (0..data.row())
                    .flat_map(|row| (0..4_u8).map(move |col| (col as usize, row as usize)))
                    .map(|(col, row)| data.get(col, row)),
            ),
        )
    }

    #[inline(always)]
    pub fn render(activity_has_occured: bool) -> Result<(), I2CError> {
        if !unsafe { INITIALIZED } {
            //nothing to do as screen is not started
            return Ok(());
        }

        if activity_has_occured {
            if !unsafe { OLED_ACTIVE } {
                Self::oled_on()?;
            } else {
                // Update timeout
                unsafe { NEXT_OLED_TIMEOUT = timer_read() + OLED_TIMEOUT };
            }
        } else {
            if !unsafe { OLED_ACTIVE } {
                //nothing to do as screen is off
                return Ok(());
            }

            if unsafe { timer_expired(NEXT_OLED_TIMEOUT) } {
                Self::oled_off()?;
                return Ok(());
            }
        }
        let mut framebuffer_as_u8 = as_u8_buf(unsafe { &mut FRAMEBUF_BINARRAY });

        for chunk in 0..OLED_BLOCK_COUNT {
            if (unsafe { DIRTY } & (1_u16 << chunk)) != 0 {
                Self::oled_draw_lines(chunk * OLED_BLOCK_ROWS, unsafe {
                    framebuffer_as_u8.extract_sized_view_unchecked::<4, OLED_BLOCK_ROWS>(
                        0,
                        chunk * OLED_BLOCK_ROWS,
                    )
                })?;
            }
        }

        unsafe { DIRTY = 0 };
        Ok(())
    }

    #[inline(always)]
    pub fn clear() {
        unsafe { FRAMEBUF_BINARRAY.backend_mut().data = [0; OLED_MATRIX_SIZE] };
        unsafe { DIRTY = ALL_DIRTY };
    }

    pub fn write_pixel(col: u8, row: u8, on: bool) {
        unsafe {
            FRAMEBUF_BINARRAY.set(col as u16, row as u16, on);
        }
        let chunk = row / OLED_BLOCK_ROWS;
        unsafe { DIRTY |= 1_u16 << chunk };
    }

    pub fn draw_char(ascii: char, offset_x: u8, offset_y: u8) {
        let char_code = ascii as u8 - b' ';

        let char_col = char_code % User::CHAR_PER_ROWS;
        let char_row = char_code / User::CHAR_PER_ROWS;
        let char_x: u16 = (char_col * User::CHAR_WIDTH) as u16;
        let char_y: u16 = (char_row * User::CHAR_HEIGHT) as u16;
        let mut buffer_view = unsafe {
            FRAMEBUF_BINARRAY
                .extract_sized_view_unchecked::<{ User::CHAR_WIDTH }, { User::CHAR_HEIGHT }>(
                    offset_x, offset_y,
                )
        };
        for col in 0..User::CHAR_WIDTH {
            for row in 0..User::CHAR_HEIGHT {
                buffer_view.set(
                    col as u16,
                    row as u16,
                    User::FONTPLATE.get(char_x + col as u16, char_y + row as u16),
                );
            }
        }
        for row in offset_y..offset_y + User::CHAR_HEIGHT {
            let block = row / OLED_BLOCK_ROWS;
            unsafe { DIRTY |= 1_u16 << block }
        }
    }

    pub fn draw_u8(mut n: u8, offset_x: u8, offset_y: u8) {
        let mut len = 0;
        let mut n2 = n;
        loop {
            len += 1;
            n2 /= 10;
            if n2 == 0 {
                break;
            }
        }
        for i in 0..len {
            Self::draw_char(
                (b'0' + n % 10) as char,
                offset_x + (len - i - 1) * User::CHAR_WIDTH,
                offset_y,
            );
            n /= 10;
        }
    }

    pub fn clear_char(offset_x: u8, offset_y: u8) {
        for cx in 0..User::CHAR_WIDTH {
            let x = offset_x + cx;
            for cy in 0..User::CHAR_HEIGHT {
                let y = offset_y + cy;
                Self::write_pixel(x, y, false);
            }
        }
    }

    pub fn draw_text<T: Iterator<Item = char>>(text: T, mut offset_x: u8, mut offset_y: u8) {
        for ascii in text {
            if offset_x + User::CHAR_WIDTH >= OLED_DISPLAY_HEIGHT {
                offset_x = 0;
                offset_y += User::CHAR_HEIGHT
            }
            Self::draw_char(ascii, offset_x, offset_y);
            offset_x += User::CHAR_WIDTH;
        }
    }

    pub fn draw_image<const N: usize>(image: QmkImage<N>, offset_x: u8, offset_y: u8) {
        let mut view = unsafe {
            FRAMEBUF_BINARRAY.extract_unsized_view_unchecked(
                offset_x,
                offset_y,
                image.width,
                image.height,
            )
        };
        let display_width = OLED_DISPLAY_HEIGHT;
        let display_height = OLED_DISPLAY_WIDTH;

        for y in 0..image.height {
            if y + offset_y >= display_height {
                break;
            }
            for x in 0..image.width {
                if x + offset_x >= display_width {
                    break;
                }
                view.set(x as u16, y as u16, image.get_pixel(x, y).unwrap());
            }
        }
        for y in offset_y..(min(display_height, offset_y + image.height)) {
            let block = y / OLED_BLOCK_ROWS;
            unsafe { DIRTY |= 1 << block }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct QmkImage<const N: usize> {
    pub width: u8,
    pub height: u8,
    pub bytes: [u8; N],
}

impl<const N: usize> QmkImage<N> {
    pub fn get_pixel(&self, x: u8, y: u8) -> Option<bool> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = y as usize * self.width as usize + x as usize;
        let byte_index = index / 8;
        let bit_index = y % 8;
        Some((self.bytes[byte_index] >> bit_index) & 1 == 1)
    }
}
