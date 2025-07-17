#![no_std]

use core::ffi::c_void;

unsafe extern "C" {
    pub fn USB_Init();
}

pub fn test() {
    // unsafe { USB_Init() };
}

#[unsafe(no_mangle)]
pub extern "C" fn CALLBACK_USB_GetDescriptor(
    value: u16,
    report: u16,
    report_size: *mut *mut c_void,
) -> u16 {
    0
}
