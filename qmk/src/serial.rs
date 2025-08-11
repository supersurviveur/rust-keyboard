pub mod shared_memory;

use core::{mem::transmute, ptr::null_mut};

use avr_base::{
    F_CPU,
    register::{EICRA, EIMSK},
};
use avr_delay::{delay_cycles, delay_us};
use keyboard_macros::config_constraints;

use crate::{
    Keyboard, QmkKeyboard,
    atomic::atomic,
    is_master,
    serial::shared_memory::{MasterSharedMemory, SlaveSharedMemory},
    timer::cycles_read,
};

const SERIAL_DELAY: u64 = 4; // in microseconds
const SERIAL_DELAY_CYCLES: u64 = SERIAL_DELAY * (F_CPU / 1_000_000); // Must be less than 255 due to many cast to u8

const _: () = if SERIAL_DELAY_CYCLES > 255 {
    panic!("SERIAL_DELAY_CYCLES must be less or equal to 255 to fit in a u8")
};

const SERIAL_DELAY_HALF: u64 = SERIAL_DELAY / 2;

const SLAVE_INT_WIDTH_US: u64 = 1;
const CHECK_CODE_SUCCESS: u8 = 0b10001;
const CHECK_CODE_ERROR: u8 = 0b01110;
const CHECK_CODE_SIZE: u8 = 5;
const MAX_TRY_COUNT: u8 = 1;

#[derive(Debug)]
pub struct SerialError;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transaction {
    /// Used to raise an error if the transaction is full of zeros (probably a pin set to low all the time)
    Reserved = 0,
    EndOfCommunication,
    SyncSlave,
    SyncMaster,
    User,
}

impl Transaction {
    #[config_constraints]
    pub fn get_receive_address<User: Keyboard>(&self) -> (*mut u8, u8) {
        match self {
            Transaction::Reserved => (null_mut(), 0),
            Transaction::EndOfCommunication => (null_mut(), 0),
            Transaction::SyncSlave => unsafe {
                (
                    SHARED_MEMORY_SLAVE,
                    size_of::<SlaveSharedMemory<User>>() as u8,
                )
            },
            Transaction::SyncMaster => unsafe {
                (
                    { SHARED_MEMORY_MASTER },
                    size_of::<MasterSharedMemory<User>>() as u8,
                )
            },
            Transaction::User => {
                todo!()
            }
        }
    }
    #[config_constraints]
    pub fn get_send_address<User: Keyboard>(&self) -> (*const u8, u8) {
        match self {
            Transaction::Reserved => (null_mut(), 0),
            Transaction::EndOfCommunication => (null_mut(), 0),
            Transaction::SyncSlave => unsafe {
                (
                    SHARED_MEMORY_SLAVE,
                    size_of::<SlaveSharedMemory<User>>() as u8,
                )
            },
            Transaction::SyncMaster => unsafe {
                (
                    { SHARED_MEMORY_MASTER },
                    size_of::<MasterSharedMemory<User>>() as u8,
                )
            },
            Transaction::User => {
                todo!()
            }
        }
    }
}

const MAX_TRANSACTION_NUMBER: u8 = Transaction::User as u8;
const TRANSACTION_BITS_SIZE: usize = MAX_TRANSACTION_NUMBER.ilog2() as usize
    + if 2u8.pow(MAX_TRANSACTION_NUMBER.ilog2()) == MAX_TRANSACTION_NUMBER {
        0
    } else {
        1
    };

/// Shared memory from slave
pub(crate) static mut SHARED_MEMORY_SLAVE: *mut u8 = null_mut();
/// Shared memory from master
pub(crate) static mut SHARED_MEMORY_MASTER: *mut u8 = null_mut();

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

    #[inline(never)]
    fn serial_sender_to_receiver(target: u8) -> u8 {
        while target.wrapping_sub(cycles_read()) as i8 >= 8 {}
        User::SOFT_SERIAL_PIN.gpio_write_pin_low();
        Self::serial_input_with_pullup();
        target.wrapping_add(SERIAL_DELAY_CYCLES as u8)
    }

    #[inline(always)]
    fn wait_target(target: u8) {
        while target.wrapping_sub(cycles_read()) as i8 >= 8 {}
    }

    /// # Safety
    /// The sender must call `Self::serial_sender_to_receiver` at the same time, to avoid having two senders on the same pin
    #[inline(never)]
    unsafe fn serial_receiver_to_sender(target: u8) -> u8 {
        while target.wrapping_sub(cycles_read()) as i8 >= 8 {}
        User::SOFT_SERIAL_PIN.gpio_write_pin_low();
        Self::serial_output();
        target.wrapping_add(SERIAL_DELAY_HALF as u8)
    }

    pub(crate) fn serial_init(&mut self) {
        unsafe { SHARED_MEMORY_MASTER = &raw mut self.master_shared_memory as *mut u8 };
        unsafe { SHARED_MEMORY_SLAVE = &raw mut self.slave_shared_memory as *mut u8 };
        if is_master() {
            Self::soft_serial_initiator_init();
        } else {
            Self::soft_serial_target_init();
        }
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

        delay_us::<{ SERIAL_DELAY * 2 }>();
        User::SOFT_SERIAL_PIN.gpio_write_pin_high();
        cycles_read().wrapping_add(SERIAL_DELAY_CYCLES as u8)
    }

    fn sync_receiver() -> u8 {
        let mut cpt: u8 = 0;
        while cpt < (SERIAL_DELAY * 2) as u8 && User::SOFT_SERIAL_PIN.gpio_read_pin() {
            cpt += 1;
            delay_cycles::<5>();
        }

        // This shouldn't hang if the target disconnects because the
        // serial line will float to high if the target does disconnect.
        while !User::SOFT_SERIAL_PIN.gpio_read_pin() {}
        cycles_read().wrapping_add(SERIAL_DELAY_CYCLES as u8 + SERIAL_DELAY_HALF as u8)
    }

    #[inline(always)]
    fn receive_sized_checked<const SIZE: usize>(has_error: &mut bool, mut target: u8) -> (u8, u8) {
        let mut transaction = 0;
        let mut parity = false;
        for _ in 0..SIZE as u8 {
            let res;
            (res, target) = Self::serial_read_pin(target);

            if res {
                transaction = (transaction << 1) | 1;
                parity ^= true;
            } else {
                transaction <<= 1;
                parity ^= false;
            }
        }
        // Receive parity bit
        let parity_sent;
        (parity_sent, target) = Self::serial_read_pin(target);
        if parity_sent != parity {
            *has_error = true;
        }
        (transaction, target)
    }
    #[inline(always)]
    fn receive_sized_unchecked<const SIZE: usize>(mut target: u8) -> (u8, u8) {
        let mut transaction = 0;
        for _ in 0..SIZE as u8 {
            let res;
            (res, target) = Self::serial_read_pin(target);

            if res {
                transaction = (transaction << 1) | 1;
            } else {
                transaction <<= 1;
            }
        }
        (transaction, target)
    }
    #[inline(always)]
    fn write_sized_checked<const SIZE: usize>(byte: u8, mut target: u8) -> u8 {
        let mut parity = false;
        let mut bit = 1 << (SIZE as u8 - 1);
        for _ in 0..SIZE as u8 {
            if byte & bit != 0 {
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
            Self::serial_high(target)
        } else {
            Self::serial_low(target)
        }
    }
    #[inline(always)]
    fn write_sized_unchecked<const SIZE: usize>(byte: u8, mut target: u8) -> u8 {
        let mut bit = 1 << (SIZE as u8 - 1);
        for _ in 0..SIZE as u8 {
            if byte & bit != 0 {
                target = Self::serial_high(target);
            } else {
                target = Self::serial_low(target);
            }

            bit >>= 1;
        }
        target
    }

    #[inline(never)]
    /// # Safety
    /// the passed pointer must point to a valid allocation of at least len bytes, wich will be overriden
    pub unsafe fn serial_read_data(try_count: u8) -> Result<bool, SerialError> {
        if try_count >= MAX_TRY_COUNT {
            return Err(SerialError);
        }
        let mut xor_check = 0;
        let mut has_error = false;
        // Sync with master
        let mut target = Self::sync_receiver();
        delay_cycles::<SERIAL_DELAY_HALF>();

        // Receive transaction byte
        let transaction;
        (transaction, target) =
            Self::receive_sized_checked::<TRANSACTION_BITS_SIZE>(&mut has_error, target);
        if has_error
            || (transaction > MAX_TRANSACTION_NUMBER || transaction == Transaction::Reserved as u8)
        {
            // This is probably an issue like disconnected keyboards, trying again is useless
            return Err(SerialError);
        }

        let transaction: Transaction = unsafe { transmute(transaction) };

        let (ptr, len) = transaction.get_receive_address();
        if transaction == Transaction::EndOfCommunication {
            return Ok(true);
        }

        if len != 0 {
            for i in 0..len {
                let byte;
                (byte, target) =
                    Self::receive_sized_checked::<{ u8::BITS as usize }>(&mut has_error, target);

                if !has_error {
                    xor_check ^= byte;
                    unsafe {
                        ptr.add(i as usize).write_volatile(byte);
                    };
                }
            }

            // Receive xor_mask byte
            let recv_xor_check;
            (recv_xor_check, target) =
                Self::receive_sized_unchecked::<{ u8::BITS as usize }>(target);

            // Send response
            let check_code = if recv_xor_check != xor_check || has_error {
                CHECK_CODE_ERROR
            } else {
                CHECK_CODE_SUCCESS
            };
            target = unsafe { Self::serial_receiver_to_sender(target) };

            target =
                Self::write_sized_unchecked::<{ CHECK_CODE_SIZE as usize }>(check_code, target);

            let _target = Self::serial_sender_to_receiver(target);

            if check_code == CHECK_CODE_ERROR {
                return unsafe { Self::serial_read_data(try_count + 1) };
            }
        }
        Ok(false)
    }

    #[inline(never)]
    pub fn serial_write_data(transaction: Transaction, try_count: u8) -> Result<(), SerialError> {
        if try_count >= MAX_TRY_COUNT {
            return Err(SerialError);
        }
        let (data, len) = transaction.get_send_address();
        let mut xor_check = 0;

        // Sync with slave
        let mut target = Self::sync_sender();

        // Send transaction byte
        target = Self::write_sized_checked::<TRANSACTION_BITS_SIZE>(transaction as u8, target);

        if len != 0 {
            for i in 0..len {
                let byte = unsafe { data.add(i as usize).read_volatile() };
                xor_check ^= byte;

                target = Self::write_sized_checked::<{ u8::BITS as usize }>(byte, target);
            }

            // Send xor_mask byte
            target = Self::write_sized_unchecked::<{ u8::BITS as usize }>(xor_check, target);

            // Receive response
            let mut check_code: u8 = 0;
            target = Self::serial_sender_to_receiver(target);

            for _ in 0..CHECK_CODE_SIZE as usize {
                let res;
                (res, target) = Self::serial_read_pin(target);

                if res {
                    check_code = (check_code << 1) | 1;
                } else {
                    check_code <<= 1;
                }
            }
            target = unsafe { Self::serial_receiver_to_sender(target) };
            let _target = Self::serial_low(target); // sync_send() / senc_recv() need raise edge

            // Loop in case of errors
            match check_code {
                // These codes are probably a success code
                0b10001 | 0b01001 | 0b10010 | 0b11001 | 0b10011 => Ok(()),
                // This is probably an issue like disconnected keyboards, trying again is useless
                0 | 0b11111111 => Err(SerialError),
                _ => Self::serial_write_data(transaction, try_count + 1),
            }
        } else {
            let _target = Self::serial_low(target); // sync_send() / senc_recv() need raise edge
            Ok(())
        }
    }

    pub fn loop_read_until_end_of_communication() -> bool {
        loop {
            match unsafe { Self::serial_read_data(0) } {
                Ok(end) => {
                    if end {
                        return false;
                    }
                }
                Err(SerialError) => {
                    unsafe { ERROR_COUNT += 1 };
                    return true;
                }
            }
        }
    }

    pub fn master_exec_transactions() {
        atomic(|| {
            Self::trigger_serial_interrupt();

            if Self::serial_write_data(Transaction::SyncMaster, 0).is_err() {
                return;
            }
            if Self::serial_write_data(Transaction::EndOfCommunication, 0).is_err() {
                return;
            }

            let mut target = Self::sync_sender();
            target = Self::serial_sender_to_receiver(target);
            Self::wait_target(target);

            if Self::loop_read_until_end_of_communication() {
                return;
            }

            let mut target = Self::sync_receiver();
            target = unsafe { Self::serial_receiver_to_sender(target) };
            Self::wait_target(target);

            // Always sync to release the slave
            Self::sync_sender();
        })
    }

    #[inline(always)]
    pub fn serial_interrupt() {
        if Self::loop_read_until_end_of_communication() {
            return;
        }

        let mut target = Self::sync_receiver();
        target = unsafe { Self::serial_receiver_to_sender(target) };
        Self::wait_target(target);

        if Self::serial_write_data(Transaction::SyncSlave, 0).is_err() {
            return;
        }
        if Self::serial_write_data(Transaction::EndOfCommunication, 0).is_err() {
            return;
        }

        let mut target = Self::sync_sender();
        target = Self::serial_sender_to_receiver(target);
        Self::wait_target(target);

        Self::sync_receiver();
    }

    #[config_constraints]
    pub fn serial_task(&mut self) {
        if is_master() {
            // Delay to avoid blocking the slave in the interrupt
            // delay_us::<1000>();
            unsafe {
                // Copy the matrix in the shared memory
                self.master_shared_memory.master_matrix = *self.current_matrix
                    [User::THIS_HAND_OFFSET as usize
                        ..User::THIS_HAND_OFFSET as usize + User::ROWS_PER_HAND as usize]
                    .as_mut_array()
                    .unwrap_unchecked();
            };
            Self::master_exec_transactions();
            // Copy the matrix from the shared memory
            unsafe {
                // Copy the matrix from the shared memory
                *self.current_matrix[User::OTHER_HAND_OFFSET as usize
                    ..User::OTHER_HAND_OFFSET as usize + User::ROWS_PER_HAND as usize]
                    .as_mut_array()
                    .unwrap_unchecked() = self.slave_shared_memory.slave_matrix;
            };
        } else {
            unsafe {
                // Copy the matrix in the shared memory
                self.slave_shared_memory.slave_matrix = *self.current_matrix[User::THIS_HAND_OFFSET
                    as usize
                    ..User::THIS_HAND_OFFSET as usize + User::ROWS_PER_HAND as usize]
                    .as_mut_array()
                    .unwrap_unchecked();
                // Copy the matrix from the shared memory
                *self.current_matrix[User::OTHER_HAND_OFFSET as usize
                    ..User::OTHER_HAND_OFFSET as usize + User::ROWS_PER_HAND as usize]
                    .as_mut_array()
                    .unwrap_unchecked() = self.master_shared_memory.master_matrix;
            };
        }
    }
}
pub static mut ERROR_COUNT: u8 = 0;
