//! This module defines USB descriptors for the keyboard and mouse.
//! It includes device, configuration, and string descriptors, as well as HID report descriptors.

use core::{ffi::c_void, ptr::null};

use crate::progmem::{self, ProgmemPtr};
use keyboard_macros::progmem;

use lufa_rs::{
    ENDPOINT_ATTR_NO_SYNC, ENDPOINT_DIR_IN, ENDPOINT_USAGE_DATA, EP_TYPE_INTERRUPT,
    HidDescriptorClassSubclassProtocol, HidDescriptorTypes, LANGUAGE_ID_ENG, NO_DESCRIPTOR,
    PackedConcreteType, USB_CONFIG_ATTR_REMOTEWAKEUP, USB_CONFIG_ATTR_RESERVED,
    UsbDescriptorClassSubclassProtocol, UsbDescriptorConfigurationHeader, UsbDescriptorDevice,
    UsbDescriptorEndpoint, UsbDescriptorHeader, UsbDescriptorInterface, UsbDescriptorString,
    UsbDescriptorTypes, UsbHidDescriptorHid, hid_descriptor_keyboard, hid_descriptor_mouse,
    usb_string_descriptor, usb_string_descriptor_array, version_bcd,
};

use crate::usb::MAX_KEYS;

const FIXED_CONTROL_ENDPOINT_SIZE: u8 = 8;
const FIXED_NUM_CONFIGURATIONS: u8 = 1;

/// Type define for the device configuration descriptor structure. This must be defined in the
/// application code, as the configuration descriptor contains several sub-descriptors which
/// vary between devices, and which describe the device's usage to the host.
pub struct UsbDescriptorConfiguration {
    pub config: UsbDescriptorConfigurationHeader,

    /// Keyboard HID Interface
    pub hid_keyboard_interface: UsbDescriptorInterface,
    pub hid_keyboard_hid: UsbHidDescriptorHid,
    pub hid_report_in_endpoint: UsbDescriptorEndpoint,

    /// Mouse HID Interface
    pub hid_mouse_interface: UsbDescriptorInterface,
    pub hid_mouse_hid: UsbHidDescriptorHid,
    pub hid_mouse_report_in_endpoint: UsbDescriptorEndpoint,
}

/// Enum for the device interface descriptor IDs within the device. Each interface descriptor
/// should have a unique ID index associated with it, which can be used to refer to the
/// interface from other descriptors.
#[repr(u8)]
pub(crate) enum InterfaceDescriptors {
    /// Keyboard interface descriptor ID
    Keyboard = 0,
    /// Mouse interface descriptor ID
    Mouse = 1,
}

/// Enum for the device string descriptor IDs within the device. Each string descriptor should
/// have a unique ID index associated with it, which can be used to refer to the string from
/// other descriptors.
#[repr(u8)]
pub enum StringDescriptors {
    /// Supported Languages string descriptor ID (must be zero).
    Language = 0,
    /// Manufacturer string ID.
    Manufacturer = 1,
    /// Keyboard product string ID.
    KeyboardProduct = 2,
}

#[doc = " \\brief Standard HID Boot Protocol Mouse Report.\n\n  Type define for a standard Boot Protocol Mouse report"]
#[repr(C, packed)]
#[derive(Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct UsbMouseReportData {
    #[doc = "< Button mask for currently pressed buttons in the mouse."]
    pub button: u8,
    #[doc = "< Current delta X movement of the mouse."]
    pub x: i8,
    #[doc = "< Current delta Y movement on the mouse."]
    pub y: i8,
    #[doc = "< Current delta V movement of the wheel."]
    pub v: i8,
    #[doc = "< Current delta H movement on the wheel."]
    pub h: i8,
}

/// Endpoint address of the Keyboard HID reporting IN endpoint.
pub const KEYBOARD_IN_ENDPOINT_ADDR: u8 = (ENDPOINT_DIR_IN | 1) as u8;

/// Endpoint address of the Mouse HID reporting IN endpoint.
pub const MOUSE_IN_ENDPOINT_ADDR: u8 = (ENDPOINT_DIR_IN | 3) as u8;

/// Size in bytes of the Keyboard HID reporting IN endpoint.
pub const HID_ENDPOINT_SIZE: u8 = 8;

/// Descripteur de périphérique
#[progmem]
static DEVICE_DESCRIPTOR: UsbDescriptorDevice = UsbDescriptorDevice {
    header: UsbDescriptorHeader {
        r#type: UsbDescriptorTypes::Device as u8,
        size: size_of::<UsbDescriptorDevice>() as u8,
    },
    usb_specification: version_bcd(3, 0, 0),
    class: UsbDescriptorClassSubclassProtocol::UsbCscpNoDeviceClass as u8,
    sub_class: UsbDescriptorClassSubclassProtocol::UsbCscpNoDeviceSubclass as u8,
    protocol: UsbDescriptorClassSubclassProtocol::UsbCscpNoDeviceProtocol as u8,
    endpoint0_size: FIXED_CONTROL_ENDPOINT_SIZE,
    vendor_id: 0xfc32,
    product_id: 0x0287,
    release_number: version_bcd(0, 0, 1),
    manufacturer_str_index: StringDescriptors::Manufacturer as u8,
    product_str_index: StringDescriptors::KeyboardProduct as u8,
    serial_num_str_index: NO_DESCRIPTOR as u8,
    number_of_configurations: FIXED_NUM_CONFIGURATIONS,
};
// Descripteur de configuration
#[progmem]
static CONFIGURATION_DESCRIPTOR: UsbDescriptorConfiguration = UsbDescriptorConfiguration {
    config: UsbDescriptorConfigurationHeader {
        header: UsbDescriptorHeader {
            r#type: UsbDescriptorTypes::Configuration as u8,
            size: size_of::<UsbDescriptorConfigurationHeader>() as u8,
        },
        total_configuration_size: size_of::<UsbDescriptorConfiguration>() as u16,
        total_interfaces: 2,
        configuration_number: 1,
        configuration_str_index: NO_DESCRIPTOR as u8,
        config_attributes: (USB_CONFIG_ATTR_RESERVED | USB_CONFIG_ATTR_REMOTEWAKEUP) as u8,
        max_power_consumption: 50, // 100 mA (2mA units)
    },
    hid_keyboard_interface: UsbDescriptorInterface {
        header: UsbDescriptorHeader {
            r#type: UsbDescriptorTypes::Interface as u8,
            size: size_of::<UsbDescriptorInterface>() as u8,
        },
        interface_number: InterfaceDescriptors::Keyboard as u8,
        alternate_setting: 0x00,
        total_endpoints: 1,
        class: HidDescriptorClassSubclassProtocol::HidCscpHidClass as u8,
        sub_class: HidDescriptorClassSubclassProtocol::HidCscpBootSubclass as u8,
        protocol: HidDescriptorClassSubclassProtocol::HidCscpKeyboardBootProtocol as u8,
        interface_str_index: NO_DESCRIPTOR as u8,
    },
    hid_keyboard_hid: UsbHidDescriptorHid {
        header: UsbDescriptorHeader {
            r#type: HidDescriptorTypes::HidHid as u8,
            size: size_of::<UsbHidDescriptorHid>() as u8,
        },
        hid_spec: version_bcd(1, 1, 1),
        country_code: 0x00,
        total_report_descriptors: 1,
        hid_report_type: HidDescriptorTypes::HidReport as u8,
        hid_report_length: KEYBOARD_DESCRIPTOR.len() as u16,
    },
    hid_report_in_endpoint: UsbDescriptorEndpoint {
        header: UsbDescriptorHeader {
            r#type: UsbDescriptorTypes::Endpoint as u8,
            size: size_of::<UsbDescriptorEndpoint>() as u8,
        },
        endpoint_address: KEYBOARD_IN_ENDPOINT_ADDR,
        attributes: (EP_TYPE_INTERRUPT | ENDPOINT_ATTR_NO_SYNC | ENDPOINT_USAGE_DATA) as u8,
        endpoint_size: HID_ENDPOINT_SIZE as u16,
        polling_interval_ms: 0x05,
    },

    hid_mouse_interface: UsbDescriptorInterface {
        header: UsbDescriptorHeader {
            size: size_of::<UsbDescriptorInterface>() as u8,
            r#type: UsbDescriptorTypes::Interface as u8,
        },

        interface_number: InterfaceDescriptors::Mouse as u8,
        alternate_setting: 0x00,

        total_endpoints: 1,

        class: HidDescriptorClassSubclassProtocol::HidCscpHidClass as u8,
        sub_class: HidDescriptorClassSubclassProtocol::HidCscpBootSubclass as u8,
        protocol: HidDescriptorClassSubclassProtocol::HidCscpMouseBootProtocol as u8,

        interface_str_index: NO_DESCRIPTOR as u8,
    },

    hid_mouse_hid: UsbHidDescriptorHid {
        header: UsbDescriptorHeader {
            size: size_of::<UsbHidDescriptorHid>() as u8,
            r#type: HidDescriptorTypes::HidHid as u8,
        },

        hid_spec: version_bcd(1, 1, 1),
        country_code: 0x00,
        total_report_descriptors: 1,
        hid_report_type: HidDescriptorTypes::HidReport as u8,
        hid_report_length: MOUSE_DESCRIPTOR.len() as u16,
    },

    hid_mouse_report_in_endpoint: UsbDescriptorEndpoint {
        header: UsbDescriptorHeader {
            size: size_of::<UsbDescriptorEndpoint>() as u8,
            r#type: UsbDescriptorTypes::Endpoint as u8,
        },

        endpoint_address: MOUSE_IN_ENDPOINT_ADDR,
        attributes: (EP_TYPE_INTERRUPT | ENDPOINT_ATTR_NO_SYNC | ENDPOINT_USAGE_DATA) as u8,
        endpoint_size: HID_ENDPOINT_SIZE as u16,
        polling_interval_ms: 0x05,
    },
};

const KEYBOARD_HID: ProgmemPtr<UsbHidDescriptorHid> =
    ProgmemPtr::new(const { &raw const CONFIGURATION_DESCRIPTOR_PROGMEM.hid_keyboard_hid });

const MOUSE_HID: ProgmemPtr<UsbHidDescriptorHid> =
    ProgmemPtr::new(const { &raw const CONFIGURATION_DESCRIPTOR_PROGMEM.hid_mouse_hid });

#[unsafe(no_mangle)]
/// Callback for retrieving USB descriptors.
///
/// # Safety
/// `descriptor_address` parameter came from LUFA, which call this function with a pointer which can always be dereferenced
pub unsafe extern "C" fn CALLBACK_USB_GetDescriptor(
    w_value: u16,
    w_index: u16,
    descriptor_address: *mut *const c_void,
) -> u16 {
    let descriptor_type = (w_value >> 8) as u8;
    let descriptor_number = (w_value & 0xFF) as u8;

    let interface_number = (w_index & 0xFF) as u8;

    let address: ProgmemPtr<()>;
    let size;

    match descriptor_type {
        c if c == UsbDescriptorTypes::Device as u8 => {
            address = DEVICE_DESCRIPTOR.as_ptr().cast();
            size = DEVICE_DESCRIPTOR.len();
        }
        c if c == UsbDescriptorTypes::Configuration as u8 => {
            address = CONFIGURATION_DESCRIPTOR.as_ptr().cast();
            size = CONFIGURATION_DESCRIPTOR.len();
        }
        c if c == UsbDescriptorTypes::String as u8 => match descriptor_number {
            code if code == StringDescriptors::Language as u8 => {
                address = LANGUAGE_STRING.as_ptr().cast();
                size = LANGUAGE_STRING.len();
            }
            code if code == StringDescriptors::Manufacturer as u8 => {
                address = MANUFACTURER_STRING.as_ptr().cast();
                size = MANUFACTURER_STRING.len();
            }
            code if code == StringDescriptors::KeyboardProduct as u8 => {
                address = KEYBOARD_PRODUCT_STRING.as_ptr().cast();
                size = KEYBOARD_PRODUCT_STRING.len();
            }
            _ => panic!(),
        },
        c if c == HidDescriptorTypes::HidHid as u8 => match interface_number {
            c if c == InterfaceDescriptors::Keyboard as u8 => {
                address = KEYBOARD_HID.cast();
                size = KEYBOARD_HID.len();
            }
            c if c == InterfaceDescriptors::Mouse as u8 => {
                address = MOUSE_HID.cast();
                size = MOUSE_HID.len();
            }
            _ => panic!(),
        },
        c if c == HidDescriptorTypes::HidReport as u8 => match interface_number {
            c if c == InterfaceDescriptors::Keyboard as u8 => {
                address = KEYBOARD_DESCRIPTOR.as_ptr().cast();
                size = KEYBOARD_DESCRIPTOR.len();
            }
            c if c == InterfaceDescriptors::Mouse as u8 => {
                address = MOUSE_DESCRIPTOR.as_ptr().cast();
                size = MOUSE_DESCRIPTOR.len();
            }
            _ => panic!(),
        },
        _ => {
            address = ProgmemPtr::new(null());
            size = 0;
        }
    }

    unsafe { *descriptor_address = address.address() as *const core::ffi::c_void };
    size as u16
}

/// Language descriptor structure.
///
/// This descriptor indicates the supported languages for string descriptors.
#[progmem]
static LANGUAGE_STRING: PackedConcreteType<UsbDescriptorString, i16, 1> =
    usb_string_descriptor_array!([LANGUAGE_ID_ENG as i16]);

/// Manufacturer descriptor string.
///
/// This string contains the manufacturer's details in human-readable form.
#[progmem]
static MANUFACTURER_STRING: PackedConcreteType<UsbDescriptorString, i16, 14> =
    usb_string_descriptor!("Surv&madcodder");

/// Product descriptor string.
///
/// This string contains the product's details in human-readable form.
#[progmem]
static KEYBOARD_PRODUCT_STRING: PackedConcreteType<UsbDescriptorString, i16, 21> =
    usb_string_descriptor!("Rust Keyboard & Mouse");

/// HID report descriptor for the keyboard.
#[progmem]
pub static KEYBOARD_DESCRIPTOR: [u8; 64] = hid_descriptor_keyboard!(MAX_KEYS);

/// HID report descriptor for the mouse.
#[progmem]
pub static MOUSE_DESCRIPTOR: [u8; 118] = hid_descriptor_mouse!();
