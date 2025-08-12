use core::ffi::c_void;

use crate::progmem::{self, ProgmemPtr};
use keyboard_macros::progmem;

use lufa_rs::{
    ENDPOINT_ATTR_NO_SYNC, ENDPOINT_DIR_IN, ENDPOINT_DIR_OUT, ENDPOINT_USAGE_DATA,
    EP_TYPE_INTERRUPT, HidDescriptorClassSubclassProtocol, HidDescriptorTypes, LANGUAGE_ID_ENG,
    NO_DESCRIPTOR, PackedConcreteType, USB_CONFIG_ATTR_RESERVED, USB_CONFIG_ATTR_SELFPOWERED,
    UsbDescriptorClassSubclassProtocol, UsbDescriptorConfigurationHeader, UsbDescriptorDevice,
    UsbDescriptorEndpoint, UsbDescriptorHeader, UsbDescriptorInterface, UsbDescriptorString,
    UsbDescriptorTypes, UsbHidDescriptorHid, hid_descriptor_keyboard, usb_string_descriptor,
    usb_string_descriptor_array, version_bcd,
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
#[progmem]
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
#[progmem]
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
        // hid_report_length: KEYBOARD_DESCRIPTOR.len() as u16,
        hid_report_length: 64,
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

const KEYBOARD_HID: ProgmemPtr<UsbHidDescriptorHid> = ProgmemPtr::new(const {&raw const CONFIGURATION_DESCRIPTOR_PROGMEM.hid_keyboard_hid});

#[unsafe(no_mangle)]
/// # Safety
/// `descriptor_address` parameter came from LUFA, which call this function with a pointer which can always be dereferenced
pub unsafe extern "C" fn CALLBACK_USB_GetDescriptor(
    w_value: u16,
    _w_index: u16,
    descriptor_address: *mut *const c_void,
) -> u16 {
    let descriptor_type = (w_value >> 8) as u8;
    let descriptor_number = (w_value & 0xFF) as u8;

    let address: ProgmemPtr<()>;
    let size;

    match descriptor_type {
        c if c == UsbDescriptorTypes::Device as u8 => {
            address = DEVICE_DESCRIPTOR.as_ptr().cast();
            size = DEVICE_DESCRIPTOR.len();
        }
        c if c == UsbDescriptorTypes::Configuration as u8 => {
            address = CONFIGURATION_DESCRIPTOR.as_ptr().cast();
            size = DEVICE_DESCRIPTOR.len();
        }
        c if c == UsbDescriptorTypes::String  as u8 => match descriptor_number {
            code if code == StringDescriptors::Language as u8 => {
                address = LANGUAGE_STRING.as_ptr().cast();
                size = LANGUAGE_STRING.len();
            }
            code if code == StringDescriptors::Manufacturer as u8 => {
                address = MANUFACTURER_STRING.as_ptr().cast();
                size = MANUFACTURER_STRING.len();
            }
            code if code == StringDescriptors::Product as u8 => {
                address = PRODUCT_STRING.as_ptr().cast();
                size = PRODUCT_STRING.len();
            }
            _ => panic!(),
        },
        c if c == HidDescriptorTypes::HidHid as u8 => {
            address = KEYBOARD_HID.cast();
            size = KEYBOARD_HID.len();
        }
        c if c == HidDescriptorTypes::HidReport as u8 => {
            address = KEYBOARD_DESCRIPTOR.as_ptr().cast();
            size = KEYBOARD_DESCRIPTOR.len();
        }
        _ => panic!(),
    }

    unsafe { *descriptor_address = address.address() as *const core::ffi::c_void };
    size as u16
}

/// Language descriptor structure. This descriptor, located in FLASH memory, is returned when the host requests
/// the string descriptor with index 0 (the first index). It is actually an array of 16-bit integers, which indicate
/// via the language ID table available at USB.org what languages the device supports for its string descriptors.
#[progmem]
static LANGUAGE_STRING: PackedConcreteType<UsbDescriptorString, i16, 1> =
    usb_string_descriptor_array!([LANGUAGE_ID_ENG as i16]);

/// Manufacturer descriptor string. This is a Unicode string containing the manufacturer's details in human readable
/// form, and is read out upon request by the host when the appropriate string ID is requested, listed in the Device
/// Descriptor.
#[progmem]
static MANUFACTURER_STRING: PackedConcreteType<UsbDescriptorString, i16, 26> =
    usb_string_descriptor!("LUFA KEYBOARD MANUFACTURER");

/// Product descriptor string. This is a Unicode string containing the product's details in human readable form,
/// and is read out upon request by the host when the appropriate string ID is requested, listed in the Device
/// Descriptor.
#[progmem]
static PRODUCT_STRING: PackedConcreteType<UsbDescriptorString, i16, 18> =
    usb_string_descriptor!("lufa rust keyboard");

#[progmem]
pub static KEYBOARD_DESCRIPTOR: [u8; 64] = hid_descriptor_keyboard!(MAX_KEYS);
