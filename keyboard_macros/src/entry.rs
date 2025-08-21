use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, Ident, ItemFn};

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

        #[unsafe(no_mangle)]
        extern "avr-interrupt" fn __vector_3() {
            qmk::QmkKeyboard::<#userkbtype>::serial_interrupt();
        }

        #[unsafe(no_mangle)]
        extern "avr-non-blocking-interrupt" fn __vector_21() {
            unsafe {qmk::timer::timer_interupt::<#userkbtype>();}
        }

        #[unsafe(no_mangle)]
        extern "C" fn main() {
            let mut kb = unsafe {QmkKeyboard::<#userkbtype>::new()};
            kb.init();
            _main_rs(&mut kb);
        }
        #main
    }
    .into()
}
