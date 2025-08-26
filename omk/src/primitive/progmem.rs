// Inspired by https://github.com/Cryptjar/avr-progmem-rs/blob/v0.4/src/raw.rs

//! Raw direct progmem access
//!
//! This module provides unsafe functions to directly access the progmem, and safe
//! ones wich are recommended to use

//! This is in particular, because having a raw `static` that is stored in the
//! progmem is very hazardous since Rust does not understand the difference
//! between the normal data memory domain and the program memory domain, and
//! allows safe code to directly access those raw progmem statics, which is
//! **undefined behavior**.
//! Safe code and macros prevent directly accessing these statics and only offer
//! dedicated accessor methods that first load the data into the normal data
//! memory domain.

#[cfg(target_arch = "avr")]
use core::arch::asm;
use core::mem::{MaybeUninit, size_of};

use crate::primitive::IndexByValue;

pub unsafe fn read_byte(p_addr: *const u8) -> u8 {
    // Only addresses below the 64 KiB limit are supported!
    // Apparently this is of no concern for architectures with true
    // 16-bit pointers.
    // TODO: switch to use the extended lpm instruction if >64k
    assert!(p_addr as usize <= u16::MAX as usize);

    // Allocate a byte for the output (actually a single register r0
    // will be used).
    let res: u8;

    // The inline assembly to read a single byte from given address
    unsafe {
        asm!(
            // Just issue the single `lpm` assembly instruction, which reads
            // implicitly indirectly the address from the Z register, and
            // stores implicitly the read value in the register 0.
            "lpm {}, Z",
            // Output is in a register
            out(reg) res,
            // Input the program memory address to read from
            in("Z") p_addr,
            // No clobber list.
        );
    }

    // Just output the read value
    res
}

/// Similar to a pointer in usage, but point to progmem instead
/// Safe to construct, unsafe to use
pub struct ProgmemPtr<T> {
    ptr: *const T,
}

/// Similar to a ref in usage, but reference progmem instead
/// unsafe to construct (prefer using the dedicated [keyboard_macros::progmem] macro)
/// but safe to use
pub struct ProgmemRef<T> {
    ptr: *const T,
}

pub struct ProgmemIterator<T> {
    ptr: ProgmemPtr<T>,
    remaining: usize,
}

impl<T> ProgmemPtr<T> {
    #[inline(always)]
    pub const fn new(ptr: *const T) -> Self {
        Self { ptr }
    }
    pub const fn address(&self) -> *const T {
        self.ptr
    }
    /// Output a single read byte
    /// Do not modify the pointer
    /// # Safety
    /// Must point to a valid location in progmem
    #[inline(always)]
    pub unsafe fn read_byte(&self) -> u8 {
        let res: u8;
        unsafe {
            asm!(
                "lpm {}, Z",
                out(reg) res,
                in("Z") self.ptr,
            );
        }
        res
    }
    /// Output a single read byte
    /// increment the pointer by one
    /// # Safety
    /// Must point to a valid location in progmem
    #[inline(always)]
    pub unsafe fn read_byte_incr(&mut self) -> u8 {
        let res: u8;
        unsafe {
            asm!(
                "lpm {}, Z+",
                out(reg) res,
                inout("Z") self.ptr,
            );
        }
        res
    }
    #[inline(always)]
    pub unsafe fn read_incr(&mut self) -> T {
        let mut res = MaybeUninit::<T>::uninit();
        let res_write_ptr = res.as_mut_ptr();
        for i in 0..size_of::<T>() {
            unsafe {
                res_write_ptr
                    .cast::<u8>()
                    .wrapping_add(i)
                    .write(self.read_byte_incr())
            };
        }
        unsafe { res.assume_init() }
    }
    #[inline(always)]
    pub const fn cast<U>(&self) -> ProgmemPtr<U> {
        ProgmemPtr {
            ptr: self.ptr.cast(),
        }
    }
    #[inline(always)]
    pub const unsafe fn iter_u8(self) -> ProgmemIterator<u8> {
        ProgmemIterator {
            ptr: self.cast(),
            remaining: (size_of::<T>()),
        }
    }
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(self) -> usize {
        size_of::<T>()
    }
}

impl<T> ProgmemRef<T> {
    /// # Safety
    /// The address must be valid (aka, realy point to a T storred in progmem)
    /// Prefer safe construction trough the use of [keyboard_macros::progmem]
    #[inline(always)]
    pub const unsafe fn new(ptr: *const T) -> Self {
        Self { ptr }
    }
    #[inline(always)]
    pub const fn as_ptr(&self) -> ProgmemPtr<T> {
        ProgmemPtr { ptr: self.ptr }
    }
    #[inline(always)]
    pub fn read(&self) -> T {
        let mut copy = self.as_ptr();
        unsafe {
            // Safe because we asserted that at ref creation
            copy.read_incr()
        }
    }
    pub const fn iter_u8(self) -> ProgmemIterator<u8> {
        ProgmemIterator {
            ptr: self.as_ptr().cast(),
            remaining: (size_of::<T>()),
        }
    }
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(self) -> usize {
        size_of::<T>()
    }
}

impl<T, const N: usize> IndexByValue<usize> for ProgmemPtr<[T; N]> {
    type Data = ProgmemPtr<T>;

    #[inline(always)]
    fn at(&self, index: usize) -> Self::Data {
        ProgmemPtr {
            ptr: (self.ptr.cast::<T>().wrapping_add(index)),
        }
    }
}

impl<T, const N: usize> IndexByValue<usize> for ProgmemRef<[T; N]> {
    type Data = ProgmemRef<T>;
    #[inline(always)]
    fn at(&self, index: usize) -> Self::Data {
        if index >= N {
            panic!();
        }
        ProgmemRef {
            ptr: (self.ptr.cast::<T>().wrapping_add(index)),
        }
    }
}
impl<T, const N: usize> ProgmemPtr<[T; N]> {
    #[inline(always)]
    #[allow(non_snake_case)]
    pub const unsafe fn iter_T(&self) -> ProgmemIterator<T> {
        ProgmemIterator {
            ptr: self.cast(),
            remaining: N,
        }
    }
}

impl<T, const N: usize> ProgmemRef<[T; N]> {
    #[inline(always)]
    #[allow(non_snake_case)]
    pub const fn iter_T(&self) -> ProgmemIterator<T> {
        ProgmemIterator {
            ptr: self.as_ptr().cast(),
            remaining: N,
        }
    }
}

impl<T> Iterator for ProgmemIterator<T> {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            unsafe { Some(self.ptr.read_incr()) }
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
    #[inline(always)]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.remaining {
            return None;
        }
        self.remaining -= n;
        self.ptr.ptr = self.ptr.ptr.wrapping_add(n);
        self.next()
    }
}

impl<T> ExactSizeIterator for ProgmemIterator<T> {}
