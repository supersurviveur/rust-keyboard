//! This module defines shared memory structures for master and slave devices in the serial communication system.
use keyboard_macros::config_constraints;
use pin_project::pin_project;

use crate::Keyboard;

/// Represents the shared memory for a master device in the serial communication system.
#[derive(Debug, Clone, Copy)]
#[pin_project(!Unpin)]
pub struct MasterSharedMemory<User: Keyboard>
where
    [(); User::ROWS_PER_HAND as usize]:,
{
    pub(crate) master_matrix: [User::MatrixRowType; User::ROWS_PER_HAND as usize],
}

impl<User: Keyboard> MasterSharedMemory<User>
where
    [(); User::ROWS_PER_HAND as usize]:,
{
    /// Creates a new instance of `MasterSharedMemory` with default values.
    pub fn new() -> Self {
        Self {
            master_matrix: [0.into(); _],
        }
    }
}

impl<User: Keyboard> Default for MasterSharedMemory<User>
where
    [(); User::ROWS_PER_HAND as usize]:,
{
    /// Provides a default instance of `MasterSharedMemory` by calling the `new` method.
    fn default() -> Self {
        Self::new()
    }
}
/// Represents the shared memory for a slave device in the serial communication system.
#[derive(Debug, Clone, Copy)]
#[pin_project(!Unpin)]
pub struct SlaveSharedMemory<User: Keyboard>
where
    [(); User::ROWS_PER_HAND as usize]:,
{
    pub(crate) slave_matrix: [User::MatrixRowType; User::ROWS_PER_HAND as usize],
}

impl<User: Keyboard> SlaveSharedMemory<User>
where
    [(); User::ROWS_PER_HAND as usize]:,
{
    /// Creates a new instance of `SlaveSharedMemory` with default values.
    pub fn new() -> Self {
        Self {
            slave_matrix: [0.into(); _],
        }
    }
}

#[config_constraints]
impl<User: Keyboard> Default for SlaveSharedMemory<User> {
    /// Provides a default instance of `SlaveSharedMemory` by calling the `new` method.
    fn default() -> Self {
        Self::new()
    }
}
