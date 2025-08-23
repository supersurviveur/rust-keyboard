//! This module provides initialization functions for the keyboard firmware.
//! It includes utilities for configuring hardware settings, such as disabling the watchdog timer.

use avr_base::register::{WDCE, WDE, WDTCSR};

use crate::atomic::atomic;

/// Disables the watchdog timer to prevent unexpected resets.
///
/// This function must be called during the initialization phase to ensure the watchdog timer does not interfere with the firmware.
pub fn disable_watchdog() {
    atomic(|| {
        WDTCSR.write(WDCE | WDE);
        WDTCSR.write(0)
    });
}
