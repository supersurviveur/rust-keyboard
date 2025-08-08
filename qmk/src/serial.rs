use avr_base::{
    F_CPU,
    register::{EICRA, EIMSK},
};
use avr_delay::{delay_cycles, delay_us};
use keyboard_macros::config_constraints;

use crate::{Keyboard, QmkKeyboard, atomic::atomic, timer::cycles_read};

const SERIAL_DELAY: u64 = 6; // in microseconds
const SERIAL_DELAY_CYCLES: u64 = SERIAL_DELAY * (F_CPU / 1_000_000); // Must be less than 255 due to many cast to u8

const _: () = if SERIAL_DELAY_CYCLES > 255 {
    panic!("SERIAL_DELAY_CYCLES must be less or equal to 255 to fit in a u8")
};

const SERIAL_DELAY_HALF: u64 = SERIAL_DELAY / 2;

const SLAVE_INT_WIDTH_US: u64 = 1;

#[repr(u8)]
pub enum Transaction {
    Test,
    StartSlaveTransactions,
}

pub static CHAINE:&[u8] = "Compiling keymap with make -r -R -f builddefs/build_keyboard.mk -s flash KEYBOARD=sofle/rev1 KEYMAP=supersurviveurRBOSE=false COLOR=true SILENT=false".as_bytes();

pub static mut RES: [u8; CHAINE.len()] = [0; CHAINE.len()];
pub static mut ERROR: bool = false;

#[config_constraints]
impl<User: Keyboard> QmkKeyboard<User> {
    #[inline(always)]
    fn serial_output() {
        User::SOFT_SERIAL_PIN.gpio_set_pin_output();
    }
    #[inline(always)]
    fn serial_input_with_pullup() {
        User::SOFT_SERIAL_PIN.gpio_set_pin_input_high();
    }
    #[inline(never)]
    fn serial_read_pin(target: u8) -> (bool, u8) {
        while target.wrapping_sub(cycles_read()) as i8 >= 0 {}
        let out = User::SOFT_SERIAL_PIN.gpio_read_pin();
        (out, target.wrapping_add(SERIAL_DELAY_CYCLES as u8))
    }
    #[inline(never)]
    fn serial_low(target: u8) -> u8 {
        while target.wrapping_sub(cycles_read()) as i8 >= 8 {}
        User::SOFT_SERIAL_PIN.gpio_write_pin_low();
        target.wrapping_add(SERIAL_DELAY_CYCLES as u8)
    }
    #[inline(never)]
    fn serial_high(target: u8) -> u8 {
        while target.wrapping_sub(cycles_read()) as i8 >= 8 {}
        User::SOFT_SERIAL_PIN.gpio_write_pin_high();
        target.wrapping_add(SERIAL_DELAY_CYCLES as u8)
    }

    pub fn soft_serial_initiator_init() {
        Self::serial_output();
        User::SOFT_SERIAL_PIN.gpio_write_pin_high();
    }

    pub fn soft_serial_target_init() {
        Self::serial_input_with_pullup();

        // Enable INT1
        EIMSK.write(EIMSK | 1 << 2);
        EICRA.write(EICRA & !(1 << 5 | 1 << 4))
    }

    fn trigger_serial_interrupt() {
        Self::serial_output();
        User::SOFT_SERIAL_PIN.gpio_write_pin_low();

        delay_us::<SLAVE_INT_WIDTH_US>();
    }

    fn sync_sender() -> u8 {
        User::SOFT_SERIAL_PIN.gpio_write_pin_low();

        delay_us::<30>();
        User::SOFT_SERIAL_PIN.gpio_write_pin_high();
        cycles_read().wrapping_add(SERIAL_DELAY_CYCLES as u8)
    }

    fn sync_receiver() -> u8 {
        let mut cpt: u8 = 0;
        while cpt < (SERIAL_DELAY * 5) as u8 && User::SOFT_SERIAL_PIN.gpio_read_pin() {
            cpt += 1;
            delay_cycles::<5>();
        }

        // This shouldn't hang if the target disconnects because the
        // serial line will float to high if the target does disconnect.
        while !User::SOFT_SERIAL_PIN.gpio_read_pin() {}
        cycles_read().wrapping_add(SERIAL_DELAY_CYCLES as u8 + SERIAL_DELAY_HALF as u8)
    }

    #[inline(never)]
    ///# Safety
    ///the passed pointer must point to a valid allocation of at least len bytes, wich will be overriden
    pub unsafe fn serial_read_data(ptr: *mut u8, len: u8) -> Result<(), (u8, u8)> {
        // Sync with master
        let mut target = Self::sync_receiver();
        delay_cycles::<SERIAL_DELAY_HALF>();
        for i in 0..len {
            let mut byte = 0;
            let mut parity = false;

            for _ in 0..u8::BITS as usize {
                // Wait SERIAL_DELAY
                let res;
                (res, target) = Self::serial_read_pin(target);

                if res {
                    byte = (byte << 1) | 1;
                    parity ^= true;
                } else {
                    byte <<= 1;
                    parity ^= false;
                }
            }
            // Receive parity bit
            let parity_sent;
            (parity_sent, target) = Self::serial_read_pin(target);
            if parity_sent != parity {
                return Err((i, byte));
            } else {
                unsafe {
                    ptr.add(i as usize).write_volatile(byte);
                };
            }
        }

        Ok(())
    }

    #[inline(never)]
    pub fn serial_write_data(data: &[u8]) {
        let len = data.len();

        // Sync with slave
        let mut target = Self::sync_sender();

        for i in 0..len {
            let octet = unsafe { data.as_ptr().add(i).read_volatile() };
            let mut parity = false;
            let mut bit = 1 << (u8::BITS - 1);

            for _ in 0..u8::BITS {
                if octet & bit != 0 {
                    target = Self::serial_high(target);
                    parity ^= true;
                } else {
                    target = Self::serial_low(target);
                    parity ^= false;
                }

                bit >>= 1;
            }
            // Send parity bit
            if parity {
                target = Self::serial_high(target);
            } else {
                target = Self::serial_low(target);
            }
        }

        let _target = Self::serial_low(target); // sync_send() / senc_recv() need raise edge
    }

    pub fn master_exec_transaction(_transaction: Transaction) {
        atomic(|| {
            Self::trigger_serial_interrupt();

            Self::serial_write_data(CHAINE);

            // Always sync to release the slave
            Self::sync_sender();
        })
    }

    #[inline(always)]
    pub fn serial_interrupt() {
        match unsafe { Self::serial_read_data({ &mut RES }.as_mut_ptr(), CHAINE.len() as u8) } {
            Ok(()) => {
                unsafe { ERROR = false };
            }
            Err((_i, _byte)) => {
                unsafe { ERROR = true };
            }
        }
        Self::sync_receiver();
    }
}
