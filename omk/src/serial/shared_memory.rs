//! This module defines shared memory structures for master and slave devices in the serial communication system.
use core::num::Wrapping;

use crate::Keyboard;

/// Represents the shared memory for a master device in the serial communication system.
#[derive(Debug, Clone, Copy)]
pub struct MasterSharedMemory<User: Keyboard> {
    pub(crate) master_matrix: [User::MatrixRowType; User::ROWS_PER_HAND],
    pub(crate) master_rotary_encoder_pulses: Wrapping<i8>,
}

impl<User: Keyboard> MasterSharedMemory<User> {
    /// Creates a new instance of `MasterSharedMemory` with default values.
    pub const fn new() -> Self {
        Self {
            master_matrix: [0.into(); _],
            master_rotary_encoder_pulses: Wrapping(0),
        }
    }
}

impl<User: Keyboard> Default for MasterSharedMemory<User> {
    /// Provides a default instance of `MasterSharedMemory` by calling the `new` method.
    fn default() -> Self {
        Self::new()
    }
}
/// Represents the shared memory for a slave device in the serial communication system.
#[derive(Debug, Clone, Copy)]
pub struct SlaveSharedMemory<User: Keyboard> {
    pub(crate) slave_matrix: [User::MatrixRowType; User::ROWS_PER_HAND],
    pub(crate) slave_rotary_encoder_pulses: Wrapping<i8>,
}

impl<User: Keyboard> SlaveSharedMemory<User> {
    /// Creates a new instance of `SlaveSharedMemory` with default values.
    pub const fn new() -> Self {
        Self {
            slave_matrix: [0.into(); _],
            slave_rotary_encoder_pulses: Wrapping(0),
        }
    }
}

impl<User: Keyboard> Default for SlaveSharedMemory<User> {
    /// Provides a default instance of `SlaveSharedMemory` by calling the `new` method.
    fn default() -> Self {
        Self::new()
    }
}
