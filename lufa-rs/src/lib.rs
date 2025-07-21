#![feature(ptr_metadata, layout_for_ptr)]
#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unsafe_op_in_unsafe_fn)]

pub mod macros;
use avr_progmem::raw::{read_byte, read_value};
pub use macros::*;

use core::{cell::LazyCell, ffi::c_void};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

unsafe extern "C" {
    pub fn SetupHardware();
    pub fn HID_Task();
}

/** Type define for the device configuration descriptor structure. This must be defined in the
 *  application code, as the configuration descriptor contains several sub-descriptors which
 *  vary between devices, and which describe the device's usage to the host.
 */
struct USB_Descriptor_Configuration {
    Config: USB_Descriptor_Configuration_Header_t,

    // Keyboard HID Interface
    HID_Interface: USB_Descriptor_Interface_t,
    HID_KeyboardHID: USB_HID_Descriptor_HID_t,
    HID_ReportINEndpoint: USB_Descriptor_Endpoint_t,
}

/** Enum for the device interface descriptor IDs within the device. Each interface descriptor
 *  should have a unique ID index associated with it, which can be used to refer to the
 *  interface from other descriptors.
 */
#[repr(u8)]
enum InterfaceDescriptors {
    /**< Keyboard interface descriptor ID */
    INTERFACE_ID_Keyboard = 0,
}

/** Enum for the device string descriptor IDs within the device. Each string descriptor should
 *  have a unique ID index associated with it, which can be used to refer to the string from
 *  other descriptors.
 */
#[repr(u8)]
enum StringDescriptors {
    /**< Supported Languages string descriptor ID (must be zero) */
    STRING_ID_Language = 0,
    /**< Manufacturer string ID */
    STRING_ID_Manufacturer = 1,
    /**< Product string ID */
    STRING_ID_Product = 2,
}

/// Endpoint address of the Keyboard HID reporting IN endpoint.
pub const KEYBOARD_EPADDR: u8 = (ENDPOINT_DIR_IN | 1) as u8;

/// Size in bytes of the Keyboard HID reporting IN endpoint
pub const KEYBOARD_EPSIZE: u8 = 8;

// Descripteur de périphérique
#[unsafe(link_section = ".progmem.data")]
#[unsafe(no_mangle)]
static DeviceDescriptor: USB_Descriptor_Device_t = USB_Descriptor_Device_t {
    Header: USB_Descriptor_Header_t {
        Type: USB_DescriptorTypes_t::DTYPE_Device as u8,
        Size: size_of::<USB_Descriptor_Device_t>() as u8,
    },
    USBSpecification: version_bcd(1, 1, 0),
    Class: USB_Descriptor_ClassSubclassProtocol_t::USB_CSCP_NoDeviceClass as u8,
    SubClass: USB_Descriptor_ClassSubclassProtocol_t::USB_CSCP_NoDeviceSubclass as u8,
    Protocol: USB_Descriptor_ClassSubclassProtocol_t::USB_CSCP_NoDeviceProtocol as u8,
    Endpoint0Size: FIXED_CONTROL_ENDPOINT_SIZE as u8,
    VendorID: 0x03EB,
    ProductID: 0x2042,
    ReleaseNumber: version_bcd(0, 0, 1),
    ManufacturerStrIndex: StringDescriptors::STRING_ID_Manufacturer as u8,
    ProductStrIndex: StringDescriptors::STRING_ID_Product as u8,
    SerialNumStrIndex: NO_DESCRIPTOR as u8,
    NumberOfConfigurations: FIXED_NUM_CONFIGURATIONS as u8,
};

// Descripteur de configuration
#[unsafe(link_section = ".progmem.data")]
#[unsafe(no_mangle)]
static ConfigurationDescriptor: USB_Descriptor_Configuration = USB_Descriptor_Configuration {
    Config: USB_Descriptor_Configuration_Header_t {
        Header: USB_Descriptor_Header_t {
            Type: USB_DescriptorTypes_t::DTYPE_Configuration as u8,
            Size: size_of::<USB_Descriptor_Configuration_Header_t>() as u8,
        },
        TotalConfigurationSize: size_of::<USB_Descriptor_Configuration>() as u16,
        TotalInterfaces: 1,
        ConfigurationNumber: 1,
        ConfigurationStrIndex: NO_DESCRIPTOR as u8,
        ConfigAttributes: (USB_CONFIG_ATTR_RESERVED | USB_CONFIG_ATTR_SELFPOWERED) as u8,
        MaxPowerConsumption: 50, // 100 mA (2mA units)
    },
    HID_Interface: USB_Descriptor_Interface_t {
        Header: USB_Descriptor_Header_t {
            Type: USB_DescriptorTypes_t::DTYPE_Interface as u8,
            Size: size_of::<USB_Descriptor_Interface_t>() as u8,
        },
        InterfaceNumber: InterfaceDescriptors::INTERFACE_ID_Keyboard as u8,
        AlternateSetting: 0x00,
        TotalEndpoints: 1,
        Class: HID_Descriptor_ClassSubclassProtocol_t::HID_CSCP_HIDClass as u8,
        SubClass: HID_Descriptor_ClassSubclassProtocol_t::HID_CSCP_BootSubclass as u8,
        Protocol: HID_Descriptor_ClassSubclassProtocol_t::HID_CSCP_KeyboardBootProtocol as u8,
        InterfaceStrIndex: NO_DESCRIPTOR as u8,
    },
    HID_KeyboardHID: USB_HID_Descriptor_HID_t {
        Header: USB_Descriptor_Header_t {
            Type: HID_DescriptorTypes_t::HID_DTYPE_HID as u8,
            Size: size_of::<USB_HID_Descriptor_HID_t>() as u8,
        },
        HIDSpec: version_bcd(1, 1, 1),
        CountryCode: 0x00,
        TotalReportDescriptors: 1,
        HIDReportType: HID_DescriptorTypes_t::HID_DTYPE_Report as u8,
        HIDReportLength: KeyboardReport.len() as u16,
    },
    HID_ReportINEndpoint: USB_Descriptor_Endpoint_t {
        Header: USB_Descriptor_Header_t {
            Type: USB_DescriptorTypes_t::DTYPE_Endpoint as u8,
            Size: size_of::<USB_Descriptor_Endpoint_t>() as u8,
        },
        EndpointAddress: KEYBOARD_EPADDR,
        Attributes: (EP_TYPE_INTERRUPT | ENDPOINT_ATTR_NO_SYNC | ENDPOINT_USAGE_DATA) as u8,
        EndpointSize: KEYBOARD_EPSIZE as u16,
        PollingIntervalMS: 0x05,
    },
};

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn CALLBACK_USB_GetDescriptor(
//     w_value: u16,
//     w_index: u16,
//     descriptor_address: *mut *const c_void,
// ) -> u16 {
//     let DescriptorType = w_value >> 8;
//     let DescriptorNumber = w_value & 0xFF;

//     let address;
//     let size;

//     match DescriptorType {
//         code if code == USB_DescriptorTypes_t::DTYPE_Device as u16 => {
//             address = &raw const DEVICE_DESCRIPTOR as *const ();
//             size = size_of::<USB_Descriptor_Device_t>() as u32;
//         }
//         code if code == USB_DescriptorTypes_t::DTYPE_Configuration as u16 => {
//             address = &raw const CONFIGURATION_DESCRIPTOR as *const ();
//             size = size_of::<USB_Descriptor_Configuration>() as u32;
//         }
//         code if code == USB_DescriptorTypes_t::DTYPE_String as u16 => match DescriptorNumber {
//             code if code == StringDescriptors::STRING_ID_Language as u16 => {
//                 address = &raw const LanguageString as *const ();
//                 size = unsafe { read_byte(&raw const LanguageString.0.Header.Size) as u32 };
//             }
//             code if code == StringDescriptors::STRING_ID_Manufacturer as u16 => {
//                 address = &raw const ManufacturerString as *const ();
//                 size = unsafe { read_value(&raw const ManufacturerString.0.Header.Size) as u32 };
//             }
//             code if code == StringDescriptors::STRING_ID_Product as u16 => {
//                 address = &raw const ProductString as *const ();
//                 size = unsafe { read_value(&raw const ProductString.0.Header.Size) as u32 };
//             }
//             _ => panic!(),
//         },
//         code if code == HID_DescriptorTypes_t::HID_DTYPE_HID as u16 => {
//             address = &raw const CONFIGURATION_DESCRIPTOR.HID_KeyboardHID as *const ();
//             size = size_of::<USB_HID_Descriptor_HID_t>() as u32;
//         }
//         code if code == HID_DescriptorTypes_t::HID_DTYPE_Report as u16 => {
//             address = KEYBOARD_DESCRIPTOR.as_ptr() as *const ();
//             size = KEYBOARD_DESCRIPTOR.len() as u32;
//         }
//         _ => panic!(),
//     }

//     unsafe { *descriptor_address = address as *const core::ffi::c_void };
//     size as u16
// }

// struct SyncLazyCell<T>(LazyCell<T>);

// unsafe impl<T> Sync for SyncLazyCell<T> {}

// /** Language descriptor structure. This descriptor, located in FLASH memory, is returned when the host requests
//  *  the string descriptor with index 0 (the first index). It is actually an array of 16-bit integers, which indicate
//  *  via the language ID table available at USB.org what languages the device supports for its string descriptors.
//  */
// static LanguageString: SyncLazyCell<&USB_Descriptor_String_t> = SyncLazyCell(LazyCell::new(|| {
//     usb_string_descriptor_array!(LANGUAGE_ID_ENG as i16)
// }));

// /** Manufacturer descriptor string. This is a Unicode string containing the manufacturer's details in human readable
//  *  form, and is read out upon request by the host when the appropriate string ID is requested, listed in the Device
//  *  Descriptor.
//  */
// static ManufacturerString: SyncLazyCell<&mut USB_Descriptor_String_t> =
//     SyncLazyCell(LazyCell::new(|| {
//         usb_string_descriptor!(&[76, 85, 70, 65, 32, 76, 105, 98, 114, 97, 114, 121])
//     }));

// /** Product descriptor string. This is a Unicode string containing the product's details in human readable form,
//  *  and is read out upon request by the host when the appropriate string ID is requested, listed in the Device
//  *  Descriptor.
//  */
// static ProductString: SyncLazyCell<&mut USB_Descriptor_String_t> =
//     SyncLazyCell(LazyCell::new(|| {
//         usb_string_descriptor!(&[
//             76, 85, 70, 65, 32, 75, 101, 121, 98, 111, 97, 114, 100, 32, 68, 101, 109, 111
//         ])
//     }));
