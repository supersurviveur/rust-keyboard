// Wrote using documentation at www.microchip.com/content/dam/mchp/documents/OTH/ProductDocuments/DataSheets/Atmel-7766-8-bit-AVR-ATmega16U4-32U4_Datasheet.pdf#M5.9.97755.SHS2..Section.Head.Sub.2.Power.Reduction.Register..PRR
//
// Permit EEPROM access

#[cfg(all(target_arch = "avr", not(doc)))]
use core::arch::asm;
use core::{
    marker::PhantomData,
    mem::{MaybeUninit, size_of},
};

use crate::primitive::IndexByValue;
use crate::{atomic::atomic, primitive::IndexByValueMut};
use avr_base::register::*;
use core::hint::unlikely;

/// Akin a *const T, but in eeprom
pub struct EepromPtr<T> {
    ptr: *const T,
}
/// Akin a &T, but in eeprom
impl<T> Clone for EepromPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}
/// Akin a *mut T, but in eeprom
pub struct EepromPtrMut<T> {
    ptr: *mut T,
}
/// Akibn a &mut T, but in eeprom
pub struct EepromRefMut<'a, T> {
    ptr: *mut T,
    _phantom: PhantomData<&'a mut T>,
}
impl<T> Copy for EepromPtr<T> {}

pub struct EepromRef<'a, T> {
    ptr: *const T,
    _phantom: PhantomData<&'a T>,
}

impl<T> EepromPtr<T> {
    pub fn new(ptr: *const T) -> Self {
        Self { ptr }
    }
    /// # Safety
    /// Must point to a correct progmem address
    pub unsafe fn read_byte(&self) -> u8 {
        #[allow(clippy::while_immutable_condition)]
        while EECR & EEPE != 0 {}
        atomic(|| {
            while unlikely(EECR & EEPE != 0) {}
            EEARH.write((self.ptr as u16 >> 8) as u8);
            EEARL.write(self.ptr as u8);
            EECR.write(EECR | EERE);
            EEDR.read()
        })
    }
    /// # Safety
    /// Must point to a T in progmem
    /// Warning: due to potential previous write failure, if T is several bytes, all repr must be correct, and potential illegal value check must be performed
    pub unsafe fn read(&self) -> T {
        let mut res = MaybeUninit::<T>::uninit();
        let mut ee_addr = *self;
        let mut res_addr: *mut u8 = res.as_mut_ptr().cast();
        for _ in 0..size_of::<T>() {
            unsafe { res_addr.write(ee_addr.read_byte()) };
            res_addr = res_addr.wrapping_byte_add(1);
            ee_addr.ptr = ee_addr.ptr.wrapping_byte_add(1);
        }
        unsafe { res.assume_init() }
    }
    pub fn cast<U>(&self) -> EepromPtr<U> {
        EepromPtr {
            ptr: self.ptr.cast(),
        }
    }
}

impl<'a, T> Clone for EepromRef<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for EepromRef<'_, T> {}

impl<'a, T> EepromRef<'a, T> {
    /// # Safety
    /// Must point to a T in progmem
    /// no live ProgmemRefMut must overlap while this one live
    pub unsafe fn new(ptr: *const T) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }
    pub fn as_ptr(&self) -> EepromPtr<T> {
        EepromPtr { ptr: self.ptr }
    }
    pub fn read_byte(&self) -> u8 {
        unsafe { self.as_ptr().read_byte() }
    }
    pub fn read(&self) -> T {
        unsafe { self.as_ptr().read() }
    }
    pub fn iter_u8<'b>(&'b self) -> EepromIterator<'b, u8>
    where
        'a: 'b,
    {
        EepromIterator {
            addr: EepromRef {
                ptr: self.ptr.cast(),
                _phantom: PhantomData,
            },
            remaining: size_of::<T>(),
        }
    }
}

impl<T> Clone for EepromPtrMut<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for EepromPtrMut<T> {}

impl<T> EepromPtrMut<T> {
    pub fn new(ptr: *mut T) -> Self {
        Self { ptr }
    }
    pub fn as_ptr(&self) -> EepromPtr<T> {
        EepromPtr { ptr: self.ptr }
    }
    /// # Safety
    /// Must point correctly
    pub unsafe fn read(&self) -> T {
        unsafe { self.as_ptr().read() }
    }
    /// # Safety
    /// Must point correctly
    pub unsafe fn read_byte(&self) -> u8 {
        unsafe { self.as_ptr().read_byte() }
    }
    /// # Safety
    /// Must point correctly
    pub unsafe fn write_byte(&self, data: u8) {
        #[allow(clippy::while_immutable_condition)]
        while EECR & EEPE != 0 {}
        atomic(|| {
            while unlikely(EECR & EEPE != 0) {}
            EEARH.write((self.ptr as u16 >> 8) as u8);
            EEARL.write(self.ptr as u8);
            EEDR.write(data);
            EECR.write(EECR | EEMPE);
            EECR.write(EEPE);
        })
    }
    /// # Safety
    /// Must Point correctly
    pub unsafe fn write(&self, data: &T) {
        let mut dataptr: *const u8 = (data as *const T).cast();
        let mut eeptr = *self;
        for _ in 0..size_of::<T>() {
            unsafe { eeptr.write_byte(*dataptr) };
            dataptr = dataptr.wrapping_byte_add(1);
            eeptr.ptr = eeptr.ptr.wrapping_byte_add(1);
        }
    }
    pub fn cast<U>(&self) -> EepromPtrMut<U> {
        EepromPtrMut {
            ptr: self.ptr.cast(),
        }
    }
}

impl<'a, T> EepromRefMut<'a, T> {
    /// # Safety
    /// Only one must be created, no other ProgmemRef or ProgmemRefMut shall live at the same time
    pub unsafe fn new(ptr: *mut T) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }
    pub fn as_mut_ptr(&self) -> EepromPtrMut<T> {
        EepromPtrMut { ptr: self.ptr }
    }
    pub fn as_ptr(&self) -> EepromPtr<T> {
        EepromPtr { ptr: self.ptr }
    }
    pub fn as_ref<'b>(&'b self) -> EepromRef<'b, T>
    where
        'a: 'b,
    {
        EepromRef {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
    }
    pub fn read(&self) -> T {
        self.as_ref().read()
    }
    pub fn read_byte(&self) -> u8 {
        self.as_ref().read_byte()
    }
    pub fn write(&mut self, data: &T) {
        unsafe { self.as_mut_ptr().write(data) };
    }
    pub fn iter_u8<'b>(&'b self) -> EepromIterator<'b, u8>
    where
        'a: 'b,
    {
        EepromIterator {
            addr: EepromRef {
                ptr: self.ptr.cast(),
                _phantom: PhantomData,
            },
            remaining: size_of::<T>(),
        }
    }
}

pub struct EepromIterator<'a, T> {
    addr: EepromRef<'a, T>,
    remaining: usize,
}

impl<'a, T> Iterator for EepromIterator<'a, T> {
    type Item = EepromRef<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            let res = self.addr;
            self.addr.ptr = self.addr.ptr.wrapping_add(1);
            Some(res)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.remaining <= n {
            None
        } else {
            self.remaining -= n;
            self.addr.ptr = self.addr.ptr.wrapping_add(n);
            self.next()
        }
    }
}

impl<'a, const N: usize, T> EepromRef<'a, [T; N]> {
    fn iter_T<'b>(&'b self) -> EepromIterator<'b, T>
    where
        'a: 'b,
    {
        EepromIterator {
            addr: EepromRef {
                ptr: self.ptr.cast(),
                _phantom: PhantomData,
            },
            remaining: N,
        }
    }
}

impl<'a, const N: usize, T> EepromRefMut<'a, [T; N]> {
    fn iter_T<'b>(&'b self) -> EepromIterator<'b, T>
    where
        'a: 'b,
    {
        EepromIterator {
            addr: EepromRef {
                ptr: self.ptr.cast(),
                _phantom: PhantomData,
            },
            remaining: N,
        }
    }
}

impl<T, const N: usize> IndexByValue<usize> for EepromPtr<[T; N]> {
    type Data = EepromPtr<T>;

    fn at(&self, index: usize) -> Self::Data {
        EepromPtr {
            ptr: self.ptr.cast::<T>().wrapping_add(index),
        }
    }
}

impl<T, const N: usize> IndexByValue<usize> for EepromPtrMut<[T; N]> {
    type Data = EepromPtrMut<T>;

    fn at(&self, index: usize) -> Self::Data {
        EepromPtrMut {
            ptr: self.ptr.cast::<T>().wrapping_add(index),
        }
    }
}

impl<'a, T, const N: usize> IndexByValue<usize> for EepromRef<'a, [T; N]> {
    type Data = EepromRef<'a, T>;

    fn at(&self, index: usize) -> Self::Data {
        if index >= N {
            panic!()
        }
        EepromRef {
            ptr: self.ptr.cast::<T>().wrapping_add(index),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T, const N: usize> IndexByValue<usize> for EepromRefMut<'a, [T; N]> {
    type Data = EepromRef<'a, T>;

    fn at(&self, index: usize) -> Self::Data {
        if index >= N {
            panic!()
        }
        EepromRef {
            ptr: self.ptr.cast::<T>().wrapping_add(index),
            _phantom: PhantomData,
        }
    }
}
