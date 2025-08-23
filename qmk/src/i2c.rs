//! This module provides functions for I2C communication.
//! It includes initialization, data transmission, and error handling utilities.

const I2C_TIMEOUT_INFINITE: u32 = u16::MAX as u32;

use crate::timer::timer_elapsed16;
use crate::timer::timer_read;
use core::ptr::read_volatile;
use core::ptr::write_volatile;

// I2C
#[allow(dead_code)]
const I2C_ACTION_READ: u8 = 0x01;
const I2C_ACTION_WRITE: u8 = 0x00;

// Registres I2C/TWI pour ATmega32u4 (Section 22.9 de la datasheet)
const TWBR: *mut u8 = 0xB8 as *mut u8; // Bit Rate Register
const TWSR: *mut u8 = 0xB9 as *mut u8; // Status Register
#[allow(dead_code)]
const TWAR: *mut u8 = 0xBA as *mut u8; // Address Register
const TWDR: *mut u8 = 0xBB as *mut u8; // Data Register
const TWCR: *mut u8 = 0xBC as *mut u8; // Control Register

// Masques de bits pour TWCR
const TWINT: u8 = 0x80; // Interrupt Flag
#[allow(dead_code)]
const TWEA: u8 = 0x40; // Enable Acknowledge
const TWSTA: u8 = 0x20; // Start Condition
const TWSTO: u8 = 0x10; // Stop Condition
#[allow(dead_code)]
const TWWC: u8 = 0x08; // Write Collision
const TWEN: u8 = 0x04; // Enable
#[allow(dead_code)]
const TWIE: u8 = 0x01; // Interrupt Enable

// Codes d'état TWSR (masqués avec TWPS)
const START: u8 = 0x08; // Start condition transmitted
const REP_START: u8 = 0x10; // Repeated start transmitted
const MT_SLA_ACK: u8 = 0x18; // SLA+W transmitted, ACK received
const MT_DATA_ACK: u8 = 0x28; // Data transmitted, ACK received

const F_SCL: u64 = 400000;
const F_CPU: u64 = 16000000;
const TWBR_VAL: u8 = (((F_CPU / F_SCL) as usize - 16) / 2) as u8;

/// Represents an error in I2C communication.
#[derive(Debug)]
pub struct I2CError();

/// Initializes the I2C interface with the specified clock frequency.
///
/// This function configures the I2C registers for communication.
pub fn i2c_init() {
    unsafe {
        write_volatile(TWSR, 0);
        write_volatile(TWBR, TWBR_VAL);
    }
}

/// Sends a START condition on the I2C bus.
///
/// # Arguments
/// * `timeout` - The timeout duration in milliseconds.
///
/// Returns `Ok(())` if the START condition was successfully sent, or an `I2CError` otherwise.
pub fn i2c_start(timeout: u16) -> Result<(), I2CError> {
    unsafe {
        // Envoyer condition START
        write_volatile(TWCR, TWINT | TWSTA | TWEN);

        // Attendre fin de transmission
        i2c_wait(timeout)?;

        // Vérifier le code d'état
        match read_volatile(TWSR) & 0xF8 {
            START | REP_START => Ok(()),
            _ => Err(I2CError()),
        }
    }
}

/// Waits for the I2C operation to complete.
///
/// # Arguments
/// * `timeout` - The timeout duration in milliseconds.
///
/// Returns `Ok(())` if the operation completed successfully, or an `I2CError` otherwise.
pub fn i2c_wait(timeout: u16) -> Result<(), I2CError> {
    unsafe {
        let timeout_timer: u32 = timer_read();
        while read_volatile(TWCR) & TWINT == 0 {
            if (timeout != I2C_TIMEOUT_INFINITE as u16)
                && (timer_elapsed16(timeout_timer) > timeout)
            {
                return Err(I2CError());
            }
        }
    }
    Ok(())
}

/// Sends a STOP condition on the I2C bus.
///
/// This function terminates the current I2C communication.
#[inline(always)]
pub fn i2c_stop() {
    unsafe {
        write_volatile(TWCR, TWINT | TWEN | TWSTO);
    }
}

/// Writes a byte of data to the I2C bus.
///
/// # Arguments
/// * `data` - The byte of data to write.
/// * `timeout` - The timeout duration in milliseconds.
///
/// Returns `Ok(())` if the data was successfully written, or an `I2CError` otherwise.
pub fn i2c_write(data: u8, timeout: u16) -> Result<(), I2CError> {
    unsafe {
        // load data into data register
        write_volatile(TWDR, data);
        // start transmission of data
        write_volatile(TWCR, TWINT | TWEN);

        i2c_wait(timeout)?;

        match read_volatile(TWSR) & 0xF8 {
            MT_SLA_ACK | MT_DATA_ACK => Result::Ok(()),
            _ => Err(I2CError()),
        }
    }
}

/// Transmits a sequence of bytes to a specific I2C address.
///
/// # Arguments
/// * `address` - The I2C address of the target device.
/// * `data` - An iterator over the bytes to transmit.
/// * `timeout` - The timeout duration in milliseconds.
///
/// Returns `Ok(())` if the transmission was successful, or an `I2CError` otherwise.
pub fn i2c_transmit<T: Iterator<Item = u8>>(
    address: u8,
    data: T,
    timeout: u16,
) -> Result<(), I2CError> {
    i2c_start(timeout)?;

    // Set address
    i2c_write(address | I2C_ACTION_WRITE, timeout)?;

    for byte in data {
        i2c_write(byte, timeout)?;
    }

    i2c_stop();
    Ok(())
}
