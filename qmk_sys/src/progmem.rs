// Taken from https://github.com/Cryptjar/avr-progmem-rs/blob/v0.4/src/raw.rs

//! Raw direct progmem access
//!
//! This module provides functions to directly access the progmem, such as
//! [read_value].
//!
//! It is recommended to use best-effort wrappers in [wrapper](crate::wrapper)
//! and [string](crate::string), which use these functions internally.
//! This is in particular, because having a raw `static` that is stored in the
//! progmem is very hazardous since Rust does not understand the difference
//! between the normal data memory domain and the program memory domain, and
//! allows safe code to directly access those raw progmem statics, which is
//! **undefined behavior**.
//! The wrapper types in [wrapper](crate::wrapper) and [string](crate::string),
//! prevent safe code from directly accessing these statics and only offer
//! dedicated accessor methods that first load the data into the normal data
//! memory domain via the function of this module.

#[cfg(all(target_arch = "avr", not(doc)))]
use core::arch::asm;
use core::mem::size_of;
use core::mem::MaybeUninit;
use core::ops::Deref;

/// Read a single byte from the progmem.
///
/// This function reads just a single byte from the program code memory domain.
/// Thus this is essentially a Rust function around the AVR `lpm` instruction.
///
/// If you need to read from an array you might use [`read_slice`] or
/// just generally for any value (including arrays) [`read_value`].
///
/// ## Example
///
/// ```
/// use avr_progmem::raw::read_byte;
/// use core::ptr::addr_of;
///
/// // This static must never be directly dereferenced/accessed!
/// // So a `let data: u8 = P_BYTE;` is Undefined Behavior!!!
/// /// Static byte stored in progmem!
/// #[link_section = ".progmem.data"]
/// static P_BYTE: u8 = b'A';
///
/// // Load the byte from progmem
/// // Here, it is sound, because due to the link_section it is indeed in the
/// // program code memory.
/// let data: u8 = unsafe { read_byte(addr_of!(P_BYTE)) };
/// assert_eq!(b'A', data);
/// ```
///
///
/// # Safety
///
/// The given point must be valid in the program domain.
/// Notice that in AVR normal pointers (to data) are into the data domain,
/// NOT the program domain.
///
/// Typically only function pointers (which make no sense here) and pointer to
/// or into statics that are defined to be stored into progmem are valid.
/// For instance, a valid progmem statics would be one, that is attributed with
/// `#[link_section = ".progmem.data"]`.
///
/// Also general Rust pointer dereferencing constraints apply (see [`core::ptr::read`]).
///
/// [`read_slice`]: fn.read_slice.html
/// [`read_value`]: fn.read_value.html
///
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

/// Read an array of type `T` from progmem into data array.
///
/// This function uses the optimized `read_asm_loop_raw` with a looped
/// assembly instead of byte-wise `read_byte` function.
///
///
/// # Safety
///
/// This call is analog to `core::ptr::copy(p_addr, out, len as usize)` thus it
/// has the same basic requirements such as both pointers must be valid for
/// dereferencing i.e. not dangling and both pointers must
/// be valid to read or write, respectively, of `len` many elements of type `T`,
/// i.e. `len * size_of::<T>()` bytes.
///
/// Additionally, `p_addr` must be a valid pointer into the program memory
/// domain. And `out` must be valid point to a writable location in the data
/// memory.
///
/// However alignment is not strictly required for AVR, since the read/write is
/// done byte-wise, but the non-AVR fallback dose actually use
/// `core::ptr::copy` and therefore the pointers must be aligned.
///
#[cfg_attr(feature = "dev", inline(never))]
unsafe fn read_asm_loop_raw<T>(p_addr: *const T, out: *mut T, len: u8) {
    // Here are the general requirements essentially required by the AVR-impl
    // However, assume, the non-AVR version is only used in tests, it makes a
    // lot of sens to ensure the AVR requirements are held up.

    // Loop head check, just return for zero iterations
    if len == 0 || size_of::<T>() == 0 {
        return;
    }

    // Get size in bytes of T
    let size_type = size_of::<T>();
    // Must not exceed 256 byte
    assert!(size_type <= u8::MAX as usize);

    // Multiply with the given length
    let size_bytes = size_type * len as usize;
    // Must still not exceed 256 byte
    assert!(size_bytes <= u8::MAX as usize);
    // Now its fine to cast down to u8
    let size_bytes = size_bytes as u8;

    // Only addresses below the 64 KiB limit are supported
    // Apparently this is of no concern for architectures with true
    // 16-bit pointers.
    // TODO: switch to use the extended lpm instruction if >64k
    assert!(p_addr as usize <= u16::MAX as usize);

    // Some dummy variables so we can define "output" for our assembly.
    // In fact, we do not have outputs, but need to modify the
    // registers, thus we just mark them as "outputs".
    let mut _a: u8;
    let mut _b: *const ();
    let mut _c: *mut ();
    let mut _d: u8;

    // A loop to read a slice of T from prog memory
    // The prog memory address (addr) is stored in the 16-bit address
    // register Z (since this is the default register for the `lpm`
    // instruction).
    // The output data memory address (out) is stored in the 16-bit
    // address register X, because Z is already used and Y seams to be
    // used otherwise or is callee-save, whatever, it emits more
    // instructions by llvm.
    //
    // This loop appears in the assembly, because it allows to exploit
    // `lpm 0, Z+` instruction that simultaneously increments the
    // pointer, and allows to write a very compact loop.
    unsafe {
        asm!(
            "
						// load value from program memory at indirect Z into temp
						// register $3 and post-increment Z by one
						lpm {1}, Z+

						// write register $3 to data memory at indirect X
						// and post-increment X by one
						st X+, {1}

						// Decrement the loop counter in register $0 (size_bytes).
						// If zero has been reached the equality flag is set.
						subi {0}, 1

						// Check whether the end has not been reached and if so jump back.
						// The end is reached if $0 (size_bytes) == 0, i.e. equality flag
						// is set.
						// Thus if equality flag is NOT set (brNE) jump back by 4
						// instruction, that are all instructions in this assembly.
						// Notice: 4 instructions = 8 Byte
						brne -8
					",
            // Some register for counting the number of bytes, gets modified
            inout(reg) size_bytes => _,
            // Some scratch register, just clobber
            out(reg) _,
            // Input address in Z, gets modified
            inout("Z") p_addr => _,
            // Output address in X, gets modified
            inout("X") out => _,
        );
    }
}

/// Read an array of type `T` from progmem into data array.
///
/// This function uses either the optimized `read_asm_loop_raw` with a
/// looped assembly instead of byte-wise `read_byte` function depending
/// whether the `lpm-asm-loop` crate feature is set.
///
///
/// # Safety
///
/// This call is analog to `core::ptr::copy(p_addr, out, len as usize)` thus it
/// has the same basic requirements such as both pointers must be valid for
/// dereferencing i.e. not dangling and both pointers must
/// be valid to read or write, respectively, of `len` many elements of type `T`,
/// i.e. `len * size_of::<T>()` bytes.
///
/// Additionally, `p_addr` must be a valid pointer into the program memory
/// domain. And `out` must be valid point to a writable location in the data
/// memory.
///
/// While the alignment is not strictly required for AVR, the non-AVR fallback
/// might be done actually use `core::ptr::copy` and therefore the pointers
/// must be aligned.
///
unsafe fn read_value_raw<T>(p_addr: *const T, out: *mut T, len: u8)
where
    T: Sized + Copy,
{
    unsafe {
        // SAFETY: The caller must ensure the validity of the pointers
        // and their domains.
        read_asm_loop_raw(p_addr, out, len)
    }
}

/// Read a single `T` from progmem and return it by value.
///
/// This function uses either a optimized assembly with loop or just a
/// byte-wise assembly function which is looped outside depending on
/// whether the `lpm-asm-loop` crate feature is set or not.
///
/// Notice that `T` might be also something like `[T, N]` so that in fact
/// entire arrays can be loaded using this function.
///
/// If you need to read just a single byte you might use [`read_byte`].
///
/// ## Example
///
/// ```
/// use avr_progmem::raw::read_value;
/// use core::ptr::addr_of;
///
/// // This static must never be directly dereferenced/accessed!
/// // So a `let data: [u8;11] = P_ARRAY;` is Undefined Behavior!!!
/// // Also notice the `*` in front of the string, because we want to store the
/// // data, not just a reference!
/// /// Static bytes stored in progmem!
/// #[link_section = ".progmem.data"]
/// static P_ARRAY: [u8;11] = *b"Hello World";
///
/// // Load the bytes from progmem
/// // Here, it is sound, because due to the link_section it is indeed in the
/// // program code memory.
/// let data: [u8;11] = unsafe { read_value(addr_of!(P_ARRAY)) };
/// assert_eq!(b"Hello World", &data);
/// ```
///
/// Also statically sized sub-arrays can be loaded using this function:
///
/// ```
/// use std::convert::TryInto;
/// use avr_progmem::raw::read_value;
///
/// /// Static bytes stored in progmem!
/// #[link_section = ".progmem.data"]
/// static P_ARRAY: [u8;11] = *b"Hello World";
///
/// // Get a sub-array reference without dereferencing it
///
/// // Make sure that we convert from &[T] directly to &[T;M] without
/// // constructing an actual [T;M], because we MAY NOT LOAD THE DATA!
/// // Also notice, that this sub-slicing does ensure that the bound are
/// // correct.
/// let slice: &[u8] = &P_ARRAY[6..];
/// let array: &[u8;5] = slice.try_into().unwrap();
///
/// // Load the bytes from progmem
/// // Here, it is sound, because due to the link_section it is indeed in the
/// // program code memory.
/// let data: [u8;5] = unsafe { read_value(array) };
/// assert_eq!(b"World", &data);
/// ```
///
/// # Panics
///
/// This function panics, if the size of the value (i.e. `size_of::<T>()`)
/// is beyond 255 bytes.
/// However, this is currently just a implementation limitation, which may
/// be lifted in the future.
///
///
/// # Safety
///
/// This call is analog to [`core::ptr::copy`] thus it
/// has the same basic requirements such as the pointer must be valid for
/// dereferencing i.e. not dangling and the pointer must
/// be valid to read one entire value of type `T`,
/// i.e. `size_of::<T>()` bytes.
///
/// Additionally, `p_addr` must be a valid pointer into the program memory
/// domain.
///
/// While the alignment is not strictly required for AVR, the non-AVR fallback
/// might be actually using `core::ptr::copy` and therefore the pointers
/// must be aligned.
///
/// [`read_byte`]: fn.read_byte.html
/// [`read_slice`]: fn.read_slice.html
///
#[cfg_attr(feature = "dev", inline(never))]
pub unsafe fn read_value<T>(p_addr: *const T) -> T
where
    T: Sized + Copy,
{
    // The use of an MaybeUninit allows us to correctly allocate the space
    // required to hold one `T`, whereas we correctly communicate that it is
    // uninitialized to the compiler.
    //
    // The alternative of using a [0u8; size_of::<T>()] is actually much more
    // cumbersome as it also removes the type inference of `read_value_raw` and
    // still requires a `transmute` in the end.
    let mut buffer = MaybeUninit::<T>::uninit();

    let size = size_of::<T>();
    // TODO add a local loop to process bigger chunks in 256 Byte blocks
    assert!(size <= u8::MAX as usize);

    let res: *mut T = buffer.as_mut_ptr();

    unsafe {
        // SAFETY: The soundness of this call is directly derived from the
        // prerequisite as defined by the Safety section of this function.
        //
        // Additionally, the use of the MaybeUninit there is also sound, because it
        // only written to and never read and not even a Rust reference is created
        // to it.
        read_value_raw(p_addr, res, 1);
    }

    unsafe {
        // SAFETY: After `read_value_raw` returned, it wrote an entire `T` into
        // the `res` pointer, which is baked by this `buffer`.
        // Thus it is now properly initialized, and this call is sound.
        buffer.assume_init()
    }
}

// Custom part

pub trait RamOrFlash<T: ?Sized> {
    fn load(&self) -> T
    where
        T: Copy;
    fn iter(&self) -> impl Iterator<Item = u8>;
}

pub struct Progmem<T: ?Sized>(pub *const T);

impl<T> RamOrFlash<T> for T {
    #[inline(always)]
    fn load(&self) -> T
    where
        T: Copy,
    {
        *self
    }

    fn iter(&self) -> impl Iterator<Item = u8> {
        RamIterator {
            value: self,
            index: 0,
        }
    }
}

impl<T> RamOrFlash<T> for Progmem<T> {
    #[inline(always)]
    fn load(&self) -> T
    where
        T: Copy,
    {
        unsafe { read_value(self.0) }
    }

    fn iter(&self) -> impl Iterator<Item = u8> {
        FlashIterator {
            start: self.0,
            index: 0,
        }
    }
}

pub struct FlashIterator<T> {
    pub start: *const T,
    pub index: usize,
}

impl<T> Iterator for FlashIterator<T> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= size_of::<T>() {
            None
        } else {
            let r = Some(unsafe { read_byte((self.start as *const u8).wrapping_add(self.index)) });
            self.index += 1;
            r
        }
    }
}
pub struct RamIterator<'a, T> {
    pub value: &'a T,
    pub index: usize,
}

impl<'a, T> Iterator for RamIterator<'a, T> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= size_of::<T>() {
            None
        } else {
            let r =
                Some(unsafe { *(self.value as *const T as *const u8).wrapping_add(self.index) });
            self.index += 1;
            r
        }
    }
}
