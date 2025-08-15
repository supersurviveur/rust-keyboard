use avr_base::register::{WDCE, WDE, WDTCSR};

use crate::atomic::atomic;

pub fn disable_watchdog() {
    atomic(|| {
        WDTCSR.write(WDCE | WDE);
        WDTCSR.write(0)
    });
}
