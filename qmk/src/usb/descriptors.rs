use core::ffi::c_void;

use lufa_rs::{
    ENDPOINT_ATTR_NO_SYNC, ENDPOINT_DIR_IN, ENDPOINT_DIR_OUT, ENDPOINT_USAGE_DATA,
    EP_TYPE_INTERRUPT, HidDescriptorClassSubclassProtocol, HidDescriptorTypes, LANGUAGE_ID_ENG,
    NO_DESCRIPTOR, PackedConcreteType, USB_CONFIG_ATTR_RESERVED, USB_CONFIG_ATTR_SELFPOWERED,
    UsbDescriptorClassSubclassProtocol, UsbDescriptorConfigurationHeader, UsbDescriptorDevice,
    UsbDescriptorEndpoint, UsbDescriptorHeader, UsbDescriptorInterface, UsbDescriptorString,
    UsbDescriptorTypes, UsbHidDescriptorHid, hid_descriptor_keyboard, usb_string_descriptor,
    usb_string_descriptor_array, version_bcd,
};
use qmk_sys::progmem::{read_byte, read_value};

use crate::usb::MAX_KEYS;

const FIXED_CONTROL_ENDPOINT_SIZE: u8 = 8;
const FIXED_NUM_CONFIGURATIONS: u8 = 1;

/// Type define for the device configuration descriptor structure. This must be defined in the
/// application code, as the configuration descriptor contains several sub-descriptors which
/// vary between devices, and which describe the device's usage to the host.
pub struct UsbDescriptorConfiguration {
    pub config: UsbDescriptorConfigurationHeader,

    /// Keyboard HID Interface
    pub hid_interface: UsbDescriptorInterface,
    pub hid_keyboard_hid: UsbHidDescriptorHid,
    pub hid_report_in_endpoint: UsbDescriptorEndpoint,
}

/// Enum for the device interface descriptor IDs within the device. Each interface descriptor
/// should have a unique ID index associated with it, which can be used to refer to the
/// interface from other descriptors.
#[repr(u8)]
enum InterfaceDescriptors {
    /// Keyboard interface descriptor ID
    Keyboard = 0,
}

/// Enum for the device string descriptor IDs within the device. Each string descriptor should
/// have a unique ID index associated with it, which can be used to refer to the string from
/// other descriptors.
#[repr(u8)]
enum StringDescriptors {
    /// Supported Languages string descriptor ID (must be zero)
    Language = 0,
    /// Manufacturer string ID
    Manufacturer = 1,
    /// Product string ID
    Product = 2,
}

/// Endpoint address of the Keyboard HID reporting IN endpoint.
pub const KEYBOARD_IN_ENDPOINT_ADDR: u8 = (ENDPOINT_DIR_IN | 1) as u8;

/// Endpoint address of the Keyboard HID reporting OUT endpoint.
pub const KEYBOARD_OUT_ENDPOINT_ADDR: u8 = (ENDPOINT_DIR_OUT | 2) as u8;

/// Size in bytes of the Keyboard HID reporting IN endpoint
pub const KEYBOARD_ENDPOINT_SIZE: u8 = 8;

/// Descripteur de périphérique
#[unsafe(link_section = ".progmem.data")]
static DEVICE_DESCRIPTOR: UsbDescriptorDevice = UsbDescriptorDevice {
    header: UsbDescriptorHeader {
        r#type: UsbDescriptorTypes::Device as u8,
        size: size_of::<UsbDescriptorDevice>() as u8,
    },
    usb_specification: version_bcd(1, 1, 0),
    class: UsbDescriptorClassSubclassProtocol::UsbCscpNoDeviceClass as u8,
    sub_class: UsbDescriptorClassSubclassProtocol::UsbCscpNoDeviceSubclass as u8,
    protocol: UsbDescriptorClassSubclassProtocol::UsbCscpNoDeviceProtocol as u8,
    endpoint0_size: FIXED_CONTROL_ENDPOINT_SIZE,
    vendor_id: 0x03EB,
    product_id: 0x2042,
    release_number: version_bcd(0, 0, 1),
    manufacturer_str_index: StringDescriptors::Manufacturer as u8,
    product_str_index: StringDescriptors::Product as u8,
    serial_num_str_index: NO_DESCRIPTOR as u8,
    number_of_configurations: FIXED_NUM_CONFIGURATIONS,
};

// Descripteur de configuration
#[unsafe(link_section = ".progmem.data")]
static CONFIGURATION_DESCRIPTOR: UsbDescriptorConfiguration = UsbDescriptorConfiguration {
    config: UsbDescriptorConfigurationHeader {
        header: UsbDescriptorHeader {
            r#type: UsbDescriptorTypes::Configuration as u8,
            size: size_of::<UsbDescriptorConfigurationHeader>() as u8,
        },
        total_configuration_size: size_of::<UsbDescriptorConfiguration>() as u16,
        total_interfaces: 1,
        configuration_number: 1,
        configuration_str_index: NO_DESCRIPTOR as u8,
        config_attributes: (USB_CONFIG_ATTR_RESERVED | USB_CONFIG_ATTR_SELFPOWERED) as u8,
        max_power_consumption: 50, // 100 mA (2mA units)
    },
    hid_interface: UsbDescriptorInterface {
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
        endpoint_size: KEYBOARD_ENDPOINT_SIZE as u16,
        polling_interval_ms: 0x05,
    },
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn CALLBACK_USB_GetDescriptor(
    w_value: u16,
    _w_index: u16,
    descriptor_address: *mut *const c_void,
) -> u16 {
    let descriptor_type = w_value >> 8;
    let descriptor_number = w_value & 0xFF;

    let address;
    let size;

    match descriptor_type {
        code if code == UsbDescriptorTypes::Device as u16 => {
            address = &raw const DEVICE_DESCRIPTOR as *const ();
            size = size_of::<UsbDescriptorDevice>() as u16;
        }
        code if code == UsbDescriptorTypes::Configuration as u16 => {
            address = &raw const CONFIGURATION_DESCRIPTOR as *const ();
            size = size_of::<UsbDescriptorConfiguration>() as u16;
        }
        code if code == UsbDescriptorTypes::String as u16 => match descriptor_number {
            code if code == StringDescriptors::Language as u16 => {
                address = &raw const LANGUAGE_STRING as *const ();
                size = unsafe { read_byte(&raw const LANGUAGE_STRING.base.header.size) as u16 };
            }
            code if code == StringDescriptors::Manufacturer as u16 => {
                address = &raw const MANUFACTURER_STRING as *const ();
                size =
                    unsafe { read_value(&raw const MANUFACTURER_STRING.base.header.size) as u16 };
            }
            code if code == StringDescriptors::Product as u16 => {
                address = &raw const PRODUCT_STRING as *const ();
                size = unsafe { read_value(&raw const PRODUCT_STRING.base.header.size) as u16 };
            }
            _ => panic!(),
        },
        code if code == HidDescriptorTypes::HidHid as u16 => {
            address = &raw const CONFIGURATION_DESCRIPTOR.hid_keyboard_hid as *const ();
            size = size_of::<UsbHidDescriptorHid>() as u16;
        }
        code if code == HidDescriptorTypes::HidReport as u16 => {
            address = KEYBOARD_DESCRIPTOR.as_ptr() as *const ();
            size = KEYBOARD_DESCRIPTOR.len() as u16;
        }
        _ => panic!(),
    }

    unsafe { *descriptor_address = address as *const core::ffi::c_void };
    size
}

/// Language descriptor structure. This descriptor, located in FLASH memory, is returned when the host requests
/// the string descriptor with index 0 (the first index). It is actually an array of 16-bit integers, which indicate
/// via the language ID table available at USB.org what languages the device supports for its string descriptors.
#[unsafe(link_section = ".progmem.data")]
static LANGUAGE_STRING: PackedConcreteType<UsbDescriptorString, i16, 1> =
    usb_string_descriptor_array!([LANGUAGE_ID_ENG as i16]);

/// Manufacturer descriptor string. This is a Unicode string containing the manufacturer's details in human readable
/// form, and is read out upon request by the host when the appropriate string ID is requested, listed in the Device
/// Descriptor.
#[unsafe(link_section = ".progmem.data")]
static MANUFACTURER_STRING: PackedConcreteType<UsbDescriptorString, i16, 26> =
    usb_string_descriptor!("LUFA KEYBOARD MANUFACTURER");

/// Product descriptor string. This is a Unicode string containing the product's details in human readable form,
/// and is read out upon request by the host when the appropriate string ID is requested, listed in the Device
/// Descriptor.
#[unsafe(link_section = ".progmem.data")]
static PRODUCT_STRING: PackedConcreteType<UsbDescriptorString, i16, 18> =
    usb_string_descriptor!("lufa rust keyboard");

#[unsafe(link_section = ".progmem.data")]
pub static KEYBOARD_DESCRIPTOR: [u8; 64] = hid_descriptor_keyboard!(MAX_KEYS);
