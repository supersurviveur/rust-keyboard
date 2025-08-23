//! # Entry Macro Implementation
//! 
//! This module provides the implementation for the `#[entry]` procedural macro. 
//! The macro is used to define the entry point of the firmware, setting up the 
//! keyboard environment and handling interrupts.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ItemFn, parse_macro_input, parse_quote};

/// Implements the `#[entry]` macro.
/// 
/// This macro sets up the entry point for the firmware. It defines the panic handler,
/// interrupt handlers, and initializes the keyboard environment before calling the
/// user-defined entry function.
/// 
/// # Arguments
/// - `args`: The type of the keyboard (e.g., `MyKeyboard`) passed as an argument to the macro.
/// - `item`: The user-defined entry function.
/// 
/// # Example
/// ```rust
/// #[entry(MyKeyboard)]
/// fn main(kb: &mut MyKeyboard) {
///     // User code here
/// }
/// ```
pub fn entry_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut main = parse_macro_input!(item as ItemFn);
    let userkbtype = parse_macro_input!(args as Ident);

    main.attrs.push(parse_quote! {#[inline(always)]});
    main.sig.ident = format_ident!("_main_rs");

    quote! {
        #[panic_handler]
        fn panic(info: &::core::panic::PanicInfo) -> ! {
            qmk::QmkKeyboard::<#userkbtype>::panic_handler(info)
        }


        static mut _THE_KEYBOARD: core::mem::MaybeUninit<qmk::QmkKeyboard<#userkbtype>> = core::mem::MaybeUninit::uninit();

        use qmk::interrupts::InterruptsHandler as _InterruptsHandler;
        
        impl _InterruptsHandler<#userkbtype> for #userkbtype {
            const KEYBOARD_PTR: *mut QmkKeyboard<#userkbtype> = const { unsafe { _THE_KEYBOARD.as_mut_ptr() } };
        }


        #[unsafe(no_mangle)]
        extern "avr-interrupt" fn __vector_3() {
            unsafe {#userkbtype::serial_interrupt();}
        }

        #[unsafe(no_mangle)]
        extern "avr-non-blocking-interrupt" fn __vector_21() {
            unsafe {#userkbtype::timer_interrupt();}
        }

        #[unsafe(no_mangle)]
        extern "C" fn main() {
            unsafe {
                _THE_KEYBOARD.as_mut_ptr().write(QmkKeyboard::<#userkbtype>::new())
            }
            let mut kb = {unsafe {core::pin::Pin::static_mut(_THE_KEYBOARD.assume_init_mut())}};
            kb.as_mut().init();
            _main_rs(kb);
        }
        #main
    }
    .into()
}
