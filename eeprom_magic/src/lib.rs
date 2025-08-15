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
///
/// Output (conceptual):
/// - `#[link_section = ".eeprom_data"]` static named `FOO_EEPROM` holding the value
/// - `const FOO: Wrapper<u32> = Wrapper::new(addr(FOO_EEPROM) - addr(__eeprom_data_start))`
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

    // Use a configurable start symbol name via cfg or env; default to __eeprom_data_start.
    // We produce: `extern "C" { static __eeprom_data_start: u8; }`
    let start_sym = syn::Ident::new("__eeprom_data_start", name.span());

    // Generate tokens
    let expanded = quote! {
        // The value that actually resides in the special section.
        #[no_mangle]
        #[link_section = ".eeprom_data"]
        #vis static #placed_name: #ty = #init;

        // Start symbol for the section, to be provided by the linker script.
        extern "C" {
            static #start_sym: u8;
        }

        // Public constant wrapper exposing the offset from section start.
        #vis const #name: eeprom::EepromRefMut<'static,#ty> = {
            // Const-evaluable address arithmetic.
            let data_ptr = (&raw const #placed_name) as usize;
            let start_ptr = unsafe { (&raw const #start_sym) as usize };
            unsafe {eeprom::EepromRefMut::<'static,#ty>::new((data_ptr - start_ptr) as *mut #ty)}
        };
    };

    TokenStream::from(expanded)
}
