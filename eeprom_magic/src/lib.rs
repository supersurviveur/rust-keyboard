extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemStatic, Visibility, parse_macro_input};

/// Attribute placed on a `static mut` initializer.
///
/// Input (example):
/// ```rust,ignore
/// #[eeprom]
/// static mut FOO: u32 = 0xDEADBEEF;
/// ```
#[proc_macro_attribute]
pub fn eeprom(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStatic);

    // Validate the pattern: `static mut NAME: Type = expr;`
    if !matches!(input.mutability, syn::StaticMutability::Mut(_)) {
        return syn::Error::new_spanned(&input, "#[eeprom] must be used on `static mut`")
            .to_compile_error()
            .into();
    }

    let vis: Visibility = input.vis.clone();
    let name = input.ident.clone();
    let ty = input.ty.clone();
    let init = input.expr.clone();

    // Derived symbols
    let placed_name = format_ident!("{}_EEPROM", name);

    // Generate tokens
    let expanded = quote! {
        // The value that actually resides in the special section.
        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".eeprom")]
        static mut #placed_name: #ty = #init;

        // Public constant wrapper exposing the offset from section start.
        #vis const #name: eeprom::EepromRefMut<'static,#ty> =
            unsafe {eeprom::EepromRefMut::<'static,#ty>::new(&raw mut #placed_name)};
    };

    TokenStream::from(expanded)
}
