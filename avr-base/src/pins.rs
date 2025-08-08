use keyboard_macros::pins;

pub const PINB_ADDRESS: u8 = 0x3;
pub const PINC_ADDRESS: u8 = 0x6;
pub const PIND_ADDRESS: u8 = 0x9;
pub const PINE_ADDRESS: u8 = 0xC;
pub const PINF_ADDRESS: u8 = 0xF;

pub const NO_PIN: Pin = Pin(!0);

pins! {
    F6, F7, B1, B3, B2, B6, C6, D2, D5, D7, E6, B4, B5
}

pub const PORT_SHIFTER: u8 = 4;
pub const ADDRESS_BASE: usize = 0x20; // 0x00 + __SFR_OFFSET = 0x20 on a atmega32u4

pub const GPIO_INPUT_PIN_DELAY: u64 = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pin(pub u8);

impl Pin {
    #[inline(always)]
    const fn pin_address(self, offset: usize) -> *mut u8 {
        (ADDRESS_BASE + (self.0 >> PORT_SHIFTER) as usize + offset) as *mut u8
    }
    #[inline(always)]
    const fn pinx_register(self) -> *mut u8 {
        self.pin_address(0)
    }
    #[inline(always)]
    const fn ddrx_register(self) -> *mut u8 {
        self.pin_address(1)
    }
    #[inline(always)]
    const fn portx_register(self) -> *mut u8 {
        self.pin_address(2)
    }

    pub fn gpio_set_pin_input(self) {
        let port = self.ddrx_register();
        let mask = 1 << (self.0 & 0xF);
        unsafe {
            let current = core::ptr::read_volatile(port);
            core::ptr::write_volatile(port, current & !mask);
        }

        let port = self.portx_register();
        unsafe {
            let current = core::ptr::read_volatile(port);
            core::ptr::write_volatile(port, current & !mask);
        }
    }

    pub fn gpio_set_pin_input_high(self) {
        let port = self.ddrx_register();
        let mask = 1 << (self.0 & 0xF);
        unsafe {
            let current = core::ptr::read_volatile(port);
            core::ptr::write_volatile(port, current & !mask);
        }

        let port = self.portx_register();
        unsafe {
            let current = core::ptr::read_volatile(port);
            core::ptr::write_volatile(port, current | mask);
        }
    }
    pub fn gpio_set_pin_output_push_pull(self) {
        let port = self.ddrx_register();
        let mask = 1 << (self.0 & 0xF);
        unsafe {
            let current = core::ptr::read_volatile(port);
            core::ptr::write_volatile(port, current | mask);
        }
    }
    #[inline(always)]
    pub fn gpio_set_pin_output(self) {
        self.gpio_set_pin_output_push_pull();
    }
    #[inline(always)]
    pub fn gpio_write_pin_high(self) {
        let port = self.portx_register();
        let mask = 1 << (self.0 & 0xF);
        unsafe {
            let current = core::ptr::read_volatile(port);
            core::ptr::write_volatile(port, current | mask);
        }
    }
    #[inline(always)]
    pub fn gpio_write_pin_low(self) {
        let port = self.portx_register();
        let mask = 1 << (self.0 & 0xF);
        unsafe {
            let current = core::ptr::read_volatile(port);
            core::ptr::write_volatile(port, current & !mask);
        }
    }
    pub fn gpio_write_pin(self, level: bool) {
        if level {
            self.gpio_write_pin_high();
        } else {
            self.gpio_write_pin_low();
        }
    }
    #[inline(always)]
    pub fn gpio_read_pin(self) -> bool {
        let port = self.pinx_register();
        let mask = 1 << (self.0 & 0xF);
        unsafe { (core::ptr::read_volatile(port) & mask) != 0 }
    }
    pub fn gpio_toggle_pin(self) {
        let port = self.portx_register();
        let mask = 1 << (self.0 & 0xF);
        unsafe {
            let current = core::ptr::read_volatile(port);
            core::ptr::write_volatile(port, current ^ mask);
        }
    }
}
