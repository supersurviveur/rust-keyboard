use core::{ffi::c_void, ptr::null_mut};

use lufa_rs::{
    EP_TYPE_INTERRUPT, Endpoint_ClearIN, Endpoint_ClearOUT, Endpoint_ClearSETUP,
    Endpoint_ClearStatusStage, Endpoint_ConfigureEndpoint, Endpoint_IsOUTReceived,
    Endpoint_IsReadWriteAllowed, Endpoint_SelectEndpoint, Endpoint_Write_8,
    Endpoint_Write_Control_Stream_LE, Endpoint_Write_Stream_LE, HID_KEYBOARD_MODIFIER_LEFTSHIFT,
    HID_KEYBOARD_SC_F, HidClassRequests, REQDIR_DEVICETOHOST, REQDIR_HOSTTODEVICE,
    REQREC_INTERFACE, REQTYPE_CLASS, USB_CONTROL_REQUEST, USB_DEVICE_STATE,
    USB_Device_EnableSOFEvents, UsbDeviceStates, UsbKeyboardReportData,
};

use crate::usb::descriptors::{
    KEYBOARD_ENDPOINT_SIZE, KEYBOARD_IN_ENDPOINT_ADDR, KEYBOARD_OUT_ENDPOINT_ADDR,
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
}

/// Event handler for the USB device Start Of Frame event.
#[unsafe(no_mangle)]
pub extern "C" fn EVENT_USB_Device_StartOfFrame() {
    unsafe {
        // One millisecond has elapsed, decrement the idle time remaining counter if
        // it has not already elapsed
        if IDLE_MS_REMAINING != 0 {
            IDLE_MS_REMAINING -= 1;
        }
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
                    IDLE_COUNT = ((USB_CONTROL_REQUEST.w_value & 0xFF00) >> 6) as u16;
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

static mut TEST: bool = false;
static mut PREV_KEYBOARD_REPORT_DATA: UsbKeyboardReportData = UsbKeyboardReportData {
    modifier: 0,
    key_code: [0; 6],
    reserved: 0,
};

pub fn create_keyboard_report(report_data: &mut UsbKeyboardReportData) {
    unsafe {
        let mut used_key_codes = 0;

        // Définition du modificateur
        report_data.modifier = HID_KEYBOARD_MODIFIER_LEFTSHIFT as u8;

        // Logique de test (alternance)
        if TEST {
            report_data.key_code[used_key_codes] = HID_KEYBOARD_SC_F as u8;
            used_key_codes += 1;
        }
        TEST = !TEST;
    }
}

pub extern "C" fn send_next_report() {
    unsafe {
        let mut keyboard_report_data = UsbKeyboardReportData::default();
        let send_report;

        // Création du rapport clavier
        create_keyboard_report(&mut keyboard_report_data);

        // Vérification de la période idle (version simplifiée)
        if IDLE_COUNT != 0 && IDLE_MS_REMAINING == 0 {
            IDLE_MS_REMAINING = IDLE_COUNT;
            send_report = true;
        } else {
            // Comparaison avec le rapport précédent
            send_report = keyboard_report_data != PREV_KEYBOARD_REPORT_DATA;
        }

        // Select the keyboard endpoint
        Endpoint_SelectEndpoint(KEYBOARD_IN_ENDPOINT_ADDR);

        // Envoi du rapport si nécessaire
        if Endpoint_IsReadWriteAllowed() && send_report {
            PREV_KEYBOARD_REPORT_DATA = keyboard_report_data;

            Endpoint_Write_Stream_LE(
                &keyboard_report_data as *const _ as *const c_void,
                size_of::<UsbKeyboardReportData>() as u16,
                null_mut(),
            );

            Endpoint_ClearIN();
        }
    }
}

pub extern "C" fn hid_task() {
    // Vérification de l'état du périphérique
    if unsafe { USB_DEVICE_STATE } != UsbDeviceStates::DeviceStateConfigured as u8 {
        return;
    }

    // Envoi du prochain rapport
    send_next_report();
}
