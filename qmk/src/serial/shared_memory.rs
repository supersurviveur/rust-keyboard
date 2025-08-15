// Use the minimum number of constraints here, otherwise cargo take a long time to compile/lint

use keyboard_macros::config_constraints;

use crate::Keyboard;


#[derive(Debug)]
pub struct MasterSharedMemory<User: Keyboard>
where
    [(); User::ROWS_PER_HAND as usize]:,
{
    pub(crate) master_matrix: [User::MatrixRowType; User::ROWS_PER_HAND as usize],
}

impl<User: Keyboard> Clone for MasterSharedMemory<User>
where
    [(); User::ROWS_PER_HAND as usize]:,
{
    #[inline]
    fn clone(&self) -> MasterSharedMemory<User> {
        *self
    }
}
impl<User: Keyboard> Copy for MasterSharedMemory<User> where [(); User::ROWS_PER_HAND as usize]: {}

#[config_constraints]
impl<User: Keyboard> Default for MasterSharedMemory<User> {
    fn default() -> Self {
        Self::new()
    }
}

#[config_constraints]
impl<User: Keyboard> MasterSharedMemory<User> {
    pub fn new() -> Self {
        Self {
            master_matrix: [0.into(); _],
        }
    }
}
#[derive(Debug)]
pub struct SlaveSharedMemory<User: Keyboard>
where
    [(); User::ROWS_PER_HAND as usize]:,
{
    pub(crate) slave_matrix: [User::MatrixRowType; User::ROWS_PER_HAND as usize],
}

impl<User: Keyboard> Clone for SlaveSharedMemory<User>
where
    [(); User::ROWS_PER_HAND as usize]:,
{
    #[inline]
    fn clone(&self) -> SlaveSharedMemory<User> {
        *self
    }
}
impl<User: Keyboard> Copy for SlaveSharedMemory<User> where [(); User::ROWS_PER_HAND as usize]: {}

#[config_constraints]
impl<User: Keyboard> Default for SlaveSharedMemory<User> {
    fn default() -> Self {
        Self::new()
    }
}

#[config_constraints]
impl<User: Keyboard> SlaveSharedMemory<User> {
    pub fn new() -> Self {
        Self {
            slave_matrix: [0.into(); _],
        }
    }
}
