use core::{ffi::c_void, ptr::null_mut};

use lufa_rs::{
    EP_TYPE_INTERRUPT, Endpoint_ClearIN, Endpoint_ClearOUT, Endpoint_ClearSETUP,
    Endpoint_ClearStatusStage, Endpoint_ConfigureEndpoint, Endpoint_IsOUTReceived,
    Endpoint_IsReadWriteAllowed, Endpoint_SelectEndpoint, Endpoint_Write_8,
    Endpoint_Write_Control_Stream_LE, Endpoint_Write_Stream_LE, HidClassRequests,
    REQDIR_DEVICETOHOST, REQDIR_HOSTTODEVICE, REQREC_INTERFACE, REQTYPE_CLASS, USB_CONTROL_REQUEST,
    USB_DEVICE_STATE, USB_Device_EnableSOFEvents, UsbDeviceStates, UsbKeyboardReportData,
};

use crate::usb::{
    MAX_KEYS,
    descriptors::{KEYBOARD_ENDPOINT_SIZE, KEYBOARD_IN_ENDPOINT_ADDR, KEYBOARD_OUT_ENDPOINT_ADDR},
};

/// Indicates what report mode the host has requested, `true` for normal HID
/// reporting mode, `false` for special boot protocol reporting mode.
static mut USING_REPORT_PROTOCOL: bool = true;

/// Current Idle period. This is set by the host via a Set Idle HID class
/// request to silence the device's reports for either the entire idle duration,
/// or until the report status changes (e.g. the user presses a key).
static mut IDLE_COUNT: u16 = 0;

/// Current Idle period remaining. When the IDLE_COUNT value is set, this tracks
/// the remaining number of idle milliseconds. This is separate to the IDLE_COUNT
/// timer and is incremented and compared as the host may request the current
/// idle period via a Get Idle HID class request, thus its value must be
/// preserved.
static mut IDLE_MS_REMAINING: u16 = 0;

/// Event handler for the USB_Connect event. This indicates that the device is
/// enumerating via the status LEDs and starts the library USB task to begin the
/// enumeration and USB management process.
#[unsafe(no_mangle)]
pub extern "C" fn EVENT_USB_Device_Connect() {
    unsafe { USING_REPORT_PROTOCOL = true };
}

/// Event handler for the USB_ConfigurationChanged event. This is fired when the
/// host sets the current configuration of the USB device after enumeration, and
/// configures the keyboard device endpoints.
#[unsafe(no_mangle)]
pub extern "C" fn EVENT_USB_Device_ConfigurationChanged() {
    let mut config_success = true;

    // Setup HID Report Endpoints
    unsafe {
        config_success &= Endpoint_ConfigureEndpoint(
            KEYBOARD_IN_ENDPOINT_ADDR,
            EP_TYPE_INTERRUPT as u8,
            KEYBOARD_ENDPOINT_SIZE as u16,
            1,
        );
        config_success &= Endpoint_ConfigureEndpoint(
            KEYBOARD_OUT_ENDPOINT_ADDR,
            EP_TYPE_INTERRUPT as u8,
            KEYBOARD_ENDPOINT_SIZE as u16,
            1,
        );
    }

    // Turn on Start-of-Frame events for tracking HID report period expiry
    unsafe { USB_Device_EnableSOFEvents() };

    if !config_success {
        panic!("Failed to configure the usb devce")
    }
}

/// Event handler for the USB device Start Of Frame event.
#[unsafe(no_mangle)]
pub extern "C" fn EVENT_USB_Device_StartOfFrame() {
    unsafe {
        // One millisecond has elapsed, decrement the idle time remaining counter if
        // it has not already elapsed
        IDLE_MS_REMAINING = IDLE_MS_REMAINING.saturating_sub(1);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn EVENT_USB_Device_ControlRequest() {
    unsafe {
        match USB_CONTROL_REQUEST.b_request {
            code if code == HidClassRequests::HidReqGetReport as u8 => {
                if USB_CONTROL_REQUEST.bm_request_type
                    == (REQDIR_DEVICETOHOST | REQTYPE_CLASS | REQREC_INTERFACE) as u8
                {
                    // Initialisation selon votre structure
                    let keyboard_report = UsbKeyboardReportData::default();

                    // CreateKeyboardReport(&mut keyboard_report);
                    Endpoint_ClearSETUP();

                    Endpoint_Write_Control_Stream_LE(
                        &keyboard_report as *const _ as *const c_void,
                        size_of::<UsbKeyboardReportData>() as u16,
                    );
                    Endpoint_ClearOUT();
                }
            }
            code if code == HidClassRequests::HidReqSetReport as u8 => {
                if USB_CONTROL_REQUEST.bm_request_type
                    == (REQDIR_HOSTTODEVICE | REQTYPE_CLASS | REQREC_INTERFACE) as u8
                {
                    Endpoint_ClearSETUP();

                    while !Endpoint_IsOUTReceived() {
                        if USB_DEVICE_STATE == UsbDeviceStates::DeviceStateUnattached as u8 {
                            return;
                        }
                    }

                    Endpoint_ClearOUT();
                    Endpoint_ClearStatusStage();
                }
            }
            code if code == HidClassRequests::HidReqGetProtocol as u8 => {
                if USB_CONTROL_REQUEST.bm_request_type
                    == (REQDIR_DEVICETOHOST | REQTYPE_CLASS | REQREC_INTERFACE) as u8
                {
                    Endpoint_ClearSETUP();
                    Endpoint_Write_8(USING_REPORT_PROTOCOL as u8);
                    Endpoint_ClearIN();
                    Endpoint_ClearStatusStage();
                }
            }
            code if code == HidClassRequests::HidReqSetProtocol as u8 => {
                if USB_CONTROL_REQUEST.bm_request_type
                    == (REQDIR_HOSTTODEVICE | REQTYPE_CLASS | REQREC_INTERFACE) as u8
                {
                    Endpoint_ClearSETUP();
                    Endpoint_ClearStatusStage();
                    USING_REPORT_PROTOCOL = USB_CONTROL_REQUEST.w_value != 0;
                }
            }

            code if code == HidClassRequests::HidReqSetIdle as u8 => {
                if USB_CONTROL_REQUEST.bm_request_type
                    == (REQDIR_HOSTTODEVICE | REQTYPE_CLASS | REQREC_INTERFACE) as u8
                {
                    Endpoint_ClearSETUP();
                    Endpoint_ClearStatusStage();
                    IDLE_COUNT = (USB_CONTROL_REQUEST.w_value & 0xFF00) >> 6;
                }
            }
            code if code == HidClassRequests::HidReqGetIdle as u8 => {
                if USB_CONTROL_REQUEST.bm_request_type
                    == (REQDIR_DEVICETOHOST | REQTYPE_CLASS | REQREC_INTERFACE) as u8
                {
                    Endpoint_ClearSETUP();
                    Endpoint_Write_8((IDLE_COUNT >> 2) as u8);
                    Endpoint_ClearIN();
                    Endpoint_ClearStatusStage();
                }
            }
            _ => {}
        }
    }
}

static mut KEYBOARD_REPORT_DATA: UsbKeyboardReportData = UsbKeyboardReportData {
    modifier: 0,
    key_code: [0; 6],
    reserved: 0,
};

static mut KEYBOARD_REPORT_DATA_UPDATED: bool = false;

pub fn add_code(code: u8) {
    let mut empty = MAX_KEYS;
    for i in 0..MAX_KEYS {
        if unsafe { KEYBOARD_REPORT_DATA.key_code[i as usize] == code } {
            return;
        } else if unsafe { KEYBOARD_REPORT_DATA.key_code[i as usize] == 0 } {
            empty = i;
        }
    }
    if empty != MAX_KEYS {
        unsafe {
            KEYBOARD_REPORT_DATA.key_code[empty as usize] = code;
            KEYBOARD_REPORT_DATA_UPDATED = true;
        }
    }
}

pub fn remove_code(code: u8) {
    for i in 0..MAX_KEYS {
        unsafe {
            if KEYBOARD_REPORT_DATA.key_code[i as usize] == code {
                KEYBOARD_REPORT_DATA.key_code[i as usize] = 0;
                KEYBOARD_REPORT_DATA_UPDATED = true;
                break;
            }
        }
    }
}
pub fn toggle_code(code: u8) {
    let mut empty = MAX_KEYS;
    for i in 0..MAX_KEYS {
        if unsafe { KEYBOARD_REPORT_DATA.key_code[i as usize] == code } {
            unsafe { KEYBOARD_REPORT_DATA.key_code[i as usize] = 0 };
            unsafe { KEYBOARD_REPORT_DATA_UPDATED = true };
            return;
        } else if unsafe { KEYBOARD_REPORT_DATA.key_code[i as usize] == 0 } {
            empty = i;
        }
    }
    if empty != MAX_KEYS {
        unsafe {
            KEYBOARD_REPORT_DATA.key_code[empty as usize] = code;
            KEYBOARD_REPORT_DATA_UPDATED = true;
        }
    }
}

pub fn send_next_report() {
    unsafe {
        let send_report = if IDLE_COUNT != 0 && IDLE_MS_REMAINING == 0 {
            IDLE_MS_REMAINING = IDLE_COUNT;
            true
        } else {
            KEYBOARD_REPORT_DATA_UPDATED
        };

        // Select the keyboard endpoint
        Endpoint_SelectEndpoint(KEYBOARD_IN_ENDPOINT_ADDR);

        if Endpoint_IsReadWriteAllowed() && send_report {
            KEYBOARD_REPORT_DATA_UPDATED = false;

            Endpoint_Write_Stream_LE(
                &KEYBOARD_REPORT_DATA as *const _ as *const c_void,
                size_of::<UsbKeyboardReportData>() as u16,
                null_mut(),
            );

            Endpoint_ClearIN();
        }
    }
}

pub fn hid_task() {
    if unsafe { USB_DEVICE_STATE } != UsbDeviceStates::DeviceStateConfigured as u8 {
        return;
    }
    send_next_report();
}
