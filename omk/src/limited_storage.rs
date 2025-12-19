use core::{array::from_fn, marker::Destruct};
union Item<T: Copy> {
    next_avail: u8,
    item: T,
}
impl<T:Copy + const Destruct> Item<T> {
    const fn next(i: usize) -> Self {
        Self {next_avail: (i+1) as u8}
    }
}

pub struct LimitedStorage<const SIZE: usize, T: Copy + const Destruct> {
    next_avail: u8,
    storage: [Item<T>; SIZE],
}

/// Error thrown when no more space available
#[derive(Debug)]
pub struct Oom();

impl<const SIZE: usize, T: Copy + const Destruct> LimitedStorage<SIZE, T> {
    pub const fn new() -> Self {
        if SIZE >= 255 {
            panic!()
        }

        Self {
            next_avail: 0,
            storage: from_fn(Item::<T>::next),
        }
    }
    /// add an element, return the storage index (needed for retrieval)
    /// or None if there is no more available room
    pub fn add(&mut self,elt: T) -> Result<u8,Oom> {
        let tmp = self.next_avail;
        if tmp == SIZE as u8 {
            Err(Oom())
        } else {
            self.next_avail =
                //Safe because this bloc is not yet allocated,
                // which mean we do use thenumber variant
                unsafe { self.storage[tmp as usize].next_avail };
            self.storage[tmp as usize].item = elt;
            Ok(tmp)
        }
    }
    /// # Safety
    /// index must be a value obtained from add, and only once
    pub unsafe fn pop(&mut self, index: u8) -> T{
        // "Safe" because of this fun requirement
        let tmp = unsafe { self.storage[index as usize].item };
        self.storage[index as usize].next_avail = self.next_avail;
        self.next_avail = index;
        tmp
    }
    /// # Safety
    /// index must be a value obtained from add, and only once
    pub unsafe fn access(&mut self,index: u8) -> &mut T {
        unsafe { &mut self.storage[index as usize].item }
    }
}

