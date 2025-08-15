use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input, parse_quote};

pub fn entry_impl(_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut main = parse_macro_input!(item as ItemFn);
    main.sig.abi = Some(parse_quote! {extern "C"});
    main.attrs.push(parse_quote! {#[unsafe(no_mangle)]});

    quote! {
        #[panic_handler]
        fn panic(info: &::core::panic::PanicInfo) -> ! {
            qmk::QmkKeyboard::<UserKeyboard>::panic_handler(info)
        }

        #[unsafe(no_mangle)]
        extern "avr-interrupt" fn __vector_3() {
            qmk::QmkKeyboard::<UserKeyboard>::serial_interrupt();
        }

        #main
    }
    .into()
}
