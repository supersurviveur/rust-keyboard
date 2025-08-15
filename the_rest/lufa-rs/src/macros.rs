pub use lufa_rs_macros;

pub const fn version_bcd(major: u16, minor: u16, revision: u16) -> u16 {
    ((major & 0xFF) << 8) | ((minor & 0x0F) << 4) | (revision & 0x0F)
}

#[repr(C, packed)]
pub struct PackedConcreteType<T, U = u8, const N: usize = 0> {
    pub base: T,
    pub array: [U; N],
}

#[macro_export]
macro_rules! usb_string_descriptor {
    ($i:expr) => {{
        let array = $crate::lufa_rs_macros::literal_to_wchar_array!($i);
        usb_string_descriptor_array!(array)
    }};
}

#[macro_export]
macro_rules! usb_string_descriptor_array {
    ($array:expr) => {{
        const PREFIX_SIZE: usize = size_of::<$crate::UsbDescriptorHeader>();
        $crate::PackedConcreteType {
            base: $crate::UsbDescriptorString {
                header: $crate::UsbDescriptorHeader {
                    size: (PREFIX_SIZE + $array.len() * size_of::<i16>()) as u8,
                    r#type: $crate::UsbDescriptorTypes::String as u8,
                },
                unicode_string: $crate::__IncompleteArrayField::new(),
            },
            array: $array,
        }
    }};
}

#[macro_export]
macro_rules! hid_descriptor_keyboard {
    ($keys:expr) => {
        $crate::lufa_rs_macros::hid_descriptor! {
            usage_page: 0x01,
            usage: 0x06,
            collection: 0x01,

            // Modifiers
            usage_page: 0x07,
            usage_minimum: 0xE0,
            usage_maximum: 0xE7,
            logical_minimum: 0x00,
            logical_maximum: 0x01,
            report_size: 1,
            report_count: 8,
            input: ($crate::HID_IOF_DATA | $crate::HID_IOF_VARIABLE | $crate::HID_IOF_ABSOLUTE) as u8,

            // Reserved
            report_count: 1,
            report_size: 8,
            input: $crate::HID_IOF_CONSTANT as u8,

            // LEDs
            usage_page: 0x08,
            usage_minimum: 0x01,
            usage_maximum: 0x05,
            report_count: 5,
            report_size: 1,
            output: ($crate::HID_IOF_DATA | $crate::HID_IOF_VARIABLE | $crate::HID_IOF_ABSOLUTE | $crate::HID_IOF_NON_VOLATILE) as u8,
            report_count: 1,
            report_size: 3,
            output: $crate::HID_IOF_CONSTANT as u8,

            // Key arrays
            logical_minimum: 0x00,
            logical_maximum: 0xFF: 16,
            usage_page: 0x07,
            usage_minimum: 0x00,
            usage_maximum: 0xFF,
            report_count: $keys,
            report_size: $keys + 2, // size_of::<KeyboardReport>()
            input: ($crate::HID_IOF_DATA | $crate::HID_IOF_ARRAY | $crate::HID_IOF_ABSOLUTE) as u8,

            end_collection
        }
    };
}
