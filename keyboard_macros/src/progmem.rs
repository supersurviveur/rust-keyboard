use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

pub(crate) fn progmem_impl(_args: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as syn::ItemStatic);

    let name = input.ident.clone();
    let ty = (*input.ty).clone();

    let new_name = format_ident!("{name}_PROGMEM");
    input.ident = new_name.clone();

    quote!(
        #[unsafe(link_section = ".progmem.data")]
        #input
        const #name: qmk_sys::progmem::Progmem<#ty> = qmk_sys::progmem::Progmem(&raw const #new_name);
    )
    .into()
}
