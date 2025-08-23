use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ItemFn, parse_macro_input, parse_quote};

pub fn entry_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut main = parse_macro_input!(item as ItemFn);
    let userkbtype = parse_macro_input!(args as Ident);
    // main.sig.abi = Some(parse_quote! {extern "C"});
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
