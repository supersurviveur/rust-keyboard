//! This module provides USB-related functionality for the keyboard firmware.
//! It includes submodules for USB descriptors and event handling.

pub mod descriptors;
pub mod events;

pub use self::descriptors::*;
pub use self::events::*;

/// Number of maximum keys pressed at the same time.
///
/// This constant defines the maximum number of keys that can be reported simultaneously.
/// It is limited to 6 due to the HID protocol constraints.
pub const MAX_KEYS: u8 = 6;
