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
#[macro_export]
macro_rules! hid_descriptor_mouse {
    () => {
        $crate::lufa_rs_macros::hid_descriptor! {
            // USAGE_PAGE (Generic Desktop)
            usage_page: 0x01,
            // USAGE (Mouse)
            usage: 0x02,
            // COLLECTION (Application)
            collection: 0x01,
            // USAGE (Mouse)
            usage: 0x02,
            // COLLECTION (Logical)
            collection: 0x02,
            // USAGE (Pointer)
            usage: 0x01,
            // COLLECTION (Physical)
            collection: 0x00,

            // ------------------------------ Buttons
            // USAGE_PAGE (Button)
            usage_page: 0x09,
            // USAGE_MINIMUM (Button 1)
            usage_minimum: 0x01,
            // USAGE_MAXIMUM (Button 5)
            usage_maximum: 0x05,
            // LOGICAL_MINIMUM (0)
            logical_minimum: 0x00,
            // LOGICAL_MAXIMUM (1)
            logical_maximum: 0x01,
            // REPORT_SIZE (1)
            report_size: 0x01,
            // REPORT_COUNT (5)
            report_count: 0x05,
            // INPUT (Data,Var,Abs)
            input: ($crate::HID_IOF_DATA | $crate::HID_IOF_VARIABLE | $crate::HID_IOF_ABSOLUTE) as u8,

            // ------------------------------ Padding
            // REPORT_SIZE (3)
            report_size: 0x03,
            // REPORT_COUNT (1)
            report_count: 0x01,
            // INPUT (Cnst,Var,Abs)
            input: $crate::HID_IOF_CONSTANT as u8,

            // ------------------------------ X,Y position
            // USAGE_PAGE (Generic Desktop)
            usage_page: 0x01,
            // USAGE (X)
            usage: 0x30,
            // USAGE (Y)
            usage: 0x31,
            // LOGICAL_MINIMUM (-127)
            logical_minimum: 0x81,
            // LOGICAL_MAXIMUM (127)
            logical_maximum: 0x7F,
            // REPORT_SIZE (8)
            report_size: 0x08,
            // REPORT_COUNT (2)
            report_count: 0x02,
            // INPUT (Data,Var,Rel)
            input: ($crate::HID_IOF_DATA | $crate::HID_IOF_VARIABLE | $crate::HID_IOF_RELATIVE) as u8,

            // ------------------------------ Vertical wheel res multiplier
            // COLLECTION (Logical)
            collection: 0x02,
            // USAGE (Resolution Multiplier)
            usage: 0x48,
            // LOGICAL_MINIMUM (0)
            logical_minimum: 0x00,
            // LOGICAL_MAXIMUM (1)
            logical_maximum: 0x01,
            // PHYSICAL_MINIMUM (1)
            physical_minimum: 0x01,
            // PHYSICAL_MAXIMUM (4)
            physical_maximum: 0x04,
            // REPORT_SIZE (2)
            report_size: 0x02,
            // REPORT_COUNT (1)
            report_count: 0x01,
            // PUSH
            push,
            // FEATURE (Data,Var,Abs)
            feature: ($crate::HID_IOF_DATA | $crate::HID_IOF_VARIABLE | $crate::HID_IOF_ABSOLUTE) as u8,

            // ------------------------------ Vertical wheel
            // USAGE (Wheel)
            usage: 0x38,
            // LOGICAL_MINIMUM (-127)
            logical_minimum: 0x81,
            // LOGICAL_MAXIMUM (127)
            logical_maximum: 0x7F,
            // PHYSICAL_MINIMUM (0) - reset physical
            physical_minimum: 0x00,
            // PHYSICAL_MAXIMUM (0)
            physical_maximum: 0x00,
            // REPORT_SIZE (8)
            report_size: 0x08,
            // INPUT (Data,Var,Rel)
            input: ($crate::HID_IOF_DATA | $crate::HID_IOF_VARIABLE | $crate::HID_IOF_RELATIVE) as u8,
            // END_COLLECTION
            end_collection,

            // ------------------------------ Horizontal wheel res multiplier
            // COLLECTION (Logical)
            collection: 0x02,
            // USAGE (Resolution Multiplier)
            usage: 0x48,
            // POP
            pop,
            // FEATURE (Data,Var,Abs)
            feature: ($crate::HID_IOF_DATA | $crate::HID_IOF_VARIABLE | $crate::HID_IOF_ABSOLUTE) as u8,

            // ------------------------------ Padding for Feature report
            // PHYSICAL_MINIMUM (0) - reset physical
            physical_minimum: 0x00,
            // PHYSICAL_MAXIMUM (0)
            physical_maximum: 0x00,
            // REPORT_SIZE (4)
            report_size: 0x04,
            // FEATURE (Cnst,Var,Abs)
            feature: $crate::HID_IOF_CONSTANT as u8,

            // ------------------------------ Horizontal wheel
            // USAGE_PAGE (Consumer Devices)
            usage_page: 0x0C,
            // USAGE (AC Pan) - 0x0238
            usage: 0x0238: 16,
            // LOGICAL_MINIMUM (-127)
            logical_minimum: 0x81,
            // LOGICAL_MAXIMUM (127)
            logical_maximum: 0x7F,
            // REPORT_SIZE (8)
            report_size: 0x08,
            // INPUT (Data,Var,Rel)
            input: ($crate::HID_IOF_DATA | $crate::HID_IOF_VARIABLE | $crate::HID_IOF_RELATIVE) as u8,
            // END_COLLECTION
            end_collection,

            // END_COLLECTION (Physical)
            end_collection,
            // END_COLLECTION (Logical)
            end_collection,
            // END_COLLECTION (Application)
            end_collection
        }
    };
}
