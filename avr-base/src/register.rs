use core::{
    ops::{BitAnd, BitOr},
    ptr::{read_volatile, write_volatile},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Register<const R: u8>();

impl<const R: u8> Register<R> {
    #[inline(always)]
    pub fn write(&self, val: u8) {
        unsafe { write_volatile(R as *mut u8, val) };
    }
    #[inline(always)]
    pub fn read(&self) -> u8 {
        unsafe { read_volatile(R as *mut u8) }
    }
}

impl<const R: u8> BitOr<u8> for Register<R> {
    type Output = u8;

    fn bitor(self, rhs: u8) -> Self::Output {
        self.read() | rhs
    }
}

impl<const R: u8> BitAnd<u8> for Register<R> {
    type Output = u8;

    fn bitand(self, rhs: u8) -> Self::Output {
        self.read() & rhs
    }
}

/// USB Controller
pub const USBCON: Register<0xD8> = Register();
/// External Interupt Mask Register
pub const EIMSK: Register<0x3D> = Register();
/// External Interupt Control Register A
pub const EICRA: Register<0x69> = Register();

/// Timer 1 Counter Low byte Register
pub const TCNT1L: Register<0x84> = Register();
/// Timer 1 Counter Control Register
pub const TCCR1B: Register<0x81> = Register();
/// Timer 1 Counter Control Register
pub const TCCR1A: Register<0x80> = Register();
/// Timer Interrupt Mask Register
pub const TIMSK0: Register<0x6E> = Register();
/// SREG
pub const SREG: Register<0x5F> = Register();
/// Output Compare Register
pub const OCR0A: Register<0x47> = Register();
/// Timer 0 Counter Control Register
pub const TCCR0B: Register<0x45> = Register();
/// Timer 0 Counter Control Register
pub const TCCR0A: Register<0x44> = Register();

/// Watch Dog Timer Control Register
pub const WDTCSR: Register<0x60> = Register();

// Registers values

/// Waveform Generation Mode
pub const WGM01: u8 = 1 << 1;
/// Timer/Counter0 Output Compare Match A Interrupt Enable
pub const OCIE0A: u8 = 1 << 1;

/// Clock Select
pub const CS00: u8 = 1 << 0;
/// Clock Select
pub const CS01: u8 = 1 << 1;
/// Clock Select
pub const CS10: u8 = 1 << 0;

/// SREG_I
pub const SREG_I: u8 = 1 << 7;

/// USB Enable
pub const USBE: u8 = 1 << 7;

/// Watch Dog system reset Enable
pub const WDE: u8 = 1 << 3;
/// Watch Dog Change Enable
pub const WDCE: u8 = 1 << 4;

/// EEPROM Registers
pub const EECR: Register<0x3F> = Register();
pub const EEARH: Register<0x42> = Register();
pub const EEARL: Register<0x41> = Register();
pub const EEDR: Register<0x40> = Register();

/// EECR (EEPROM Control Register) bits
pub const EEPM1: u8 = 1 << 5;
pub const EEPM0: u8 = 1 << 4;
pub const EERIE: u8 = 1 << 3;
pub const EEMPE: u8 = 1 << 2;
pub const EEPE: u8 = 1 << 1;
pub const EERE: u8 = 1 << 0;
