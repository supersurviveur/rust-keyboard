use keyboard_macros::hid_keyboard_descriptor;

pub const fn version_bcd(major: u16, minor: u16, revision: u16) -> u16 {
    ((major & 0xFF) << 8) | ((minor & 0x0F) << 4) | (revision & 0x0F)
}

#[macro_export]
macro_rules! usb_string_descriptor {
    ($i:expr) => {
        {
            const prefix_size: usize = size_of::<USB_Descriptor_Header_t>();
            let mut res = [0u8; (prefix_size + $i.len())];
            let prefix: &mut USB_Descriptor_String_t = unsafe { core::mem::transmute(TryInto::<&mut [u8; prefix_size]>::try_into(&mut res[0..prefix_size])) };
            *prefix = USB_Descriptor_String_t {
                Header: USB_Descriptor_Header_t {
                    Size: (prefix_size + $i.len() - 2) as u8,
                    Type: USB_DescriptorTypes_t::DTYPE_String as u8
                },
                UnicodeString: __IncompleteArrayField::new()
            };
            unsafe {
                prefix.UnicodeString.as_mut_slice($i.len()).copy_from_slice($i);
            }
            prefix
        }
    };
}

#[macro_export]
macro_rules! usb_string_descriptor_array {
    ($($args:expr),*) => {
        {
            const prefix_size: usize = size_of::<USB_Descriptor_Header_t>();
            let mut res = [0u8; (prefix_size + [$($args),*].len())];
            let prefix: &mut USB_Descriptor_String_t = unsafe { core::mem::transmute(TryInto::<&mut [u8; prefix_size]>::try_into(&mut res[0..prefix_size])) };
            *prefix = USB_Descriptor_String_t {
                Header: USB_Descriptor_Header_t {
                    Size: (prefix_size + [$($args),*].len()) as u8,
                    Type: USB_DescriptorTypes_t::DTYPE_String as u8
                },
                UnicodeString: __IncompleteArrayField::new()
            };
            unsafe {
                prefix.UnicodeString.as_mut_slice([$($args),*].len()).copy_from_slice(&[$($args),*]);
            }
            prefix
        }
    }
}

// /** \hideinitializer
//  *  A list of HID report item array elements that describe a typical HID USB keyboard. The resulting report descriptor
//  *  is compatible with \ref USB_KeyboardReport_Data_t when \c MaxKeys is equal to 6. For other values, the report will
//  *  be structured according to the following layout:
//  *
//  *  \code
//  *  struct
//  *  {
//  *      uint8_t Modifier; // Keyboard modifier byte indicating pressed modifier keys (\c HID_KEYBOARD_MODIFER_* masks)
//  *      uint8_t Reserved; // Reserved for OEM use, always set to 0.
//  *      uint8_t KeyCode[MaxKeys]; // Length determined by the number of keys that can be reported
//  *  } Keyboard_Report;
//  *  \endcode
//  *
//  *  \param[in] MaxKeys  Number of simultaneous keys that can be reported at the one time (8-bit).
//  */
// pub const fn hid_descriptor_keyboard(max_keys: u8) {
//     // HID_RI_USAGE_PAGE(8, 0x01),
//     // HID_RI_USAGE(8, 0x06),
//     // HID_RI_COLLECTION(8, 0x01),
//     // 	HID_RI_USAGE_PAGE(8, 0x07),
//     // 	HID_RI_USAGE_MINIMUM(8, 0xE0),
//     // 	HID_RI_USAGE_MAXIMUM(8, 0xE7),
//     // 	HID_RI_LOGICAL_MINIMUM(8, 0x00),
//     // 	HID_RI_LOGICAL_MAXIMUM(8, 0x01),
//     // 	HID_RI_REPORT_SIZE(8, 0x01),
//     // 	HID_RI_REPORT_COUNT(8, 0x08),
//     // 	HID_RI_INPUT(8, HID_IOF_DATA | HID_IOF_VARIABLE | HID_IOF_ABSOLUTE),
//     // 	HID_RI_REPORT_COUNT(8, 0x01),
//     // 	HID_RI_REPORT_SIZE(8, 0x08),
//     // 	HID_RI_INPUT(8, HID_IOF_CONSTANT),
//     // 	HID_RI_USAGE_PAGE(8, 0x08),
//     // 	HID_RI_USAGE_MINIMUM(8, 0x01),
//     // 	HID_RI_USAGE_MAXIMUM(8, 0x05),
//     // 	HID_RI_REPORT_COUNT(8, 0x05),
//     // 	HID_RI_REPORT_SIZE(8, 0x01),
//     // 	HID_RI_OUTPUT(8, HID_IOF_DATA | HID_IOF_VARIABLE | HID_IOF_ABSOLUTE | HID_IOF_NON_VOLATILE),
//     // 	HID_RI_REPORT_COUNT(8, 0x01),
//     // 	HID_RI_REPORT_SIZE(8, 0x03),
//     // 	HID_RI_OUTPUT(8, HID_IOF_CONSTANT),
//     // 	HID_RI_LOGICAL_MINIMUM(8, 0x00),
//     // 	HID_RI_LOGICAL_MAXIMUM(16, 0xFF),
//     // 	HID_RI_USAGE_PAGE(8, 0x07),
//     // 	HID_RI_USAGE_MINIMUM(8, 0x00),
//     // 	HID_RI_USAGE_MAXIMUM(8, 0xFF),
//     // 	HID_RI_REPORT_COUNT(8, MaxKeys),
//     // 	HID_RI_REPORT_SIZE(8, 0x08),
//     // 	HID_RI_INPUT(8, HID_IOF_DATA | HID_IOF_ARRAY | HID_IOF_ABSOLUTE),
//     // HID_RI_END_COLLECTION(0)
// }

#[unsafe(no_mangle)]
pub static KeyboardReport: &[u8] = &hid_keyboard_descriptor!(6);
#[unsafe(no_mangle)]
pub static KeyboardReport_size: usize = KeyboardReport.len();
