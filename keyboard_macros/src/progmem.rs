//! # Progmem Macro Implementation
//! 
//! This module provides the implementation for the `progmem` procedural macro.
//! The macro is used to place static variables in the `.progmem.data` section of memory, which is typically used for read-only program memory in embedded systems.
//! 
//! ## Usage
//! The `progmem` macro can be applied to static variables to store them in program memory.
//! 
//! ## Example
//! ```rust
//! progmem! {
//!     static FOO: [u8; 4] = [1, 2, 3, 4];
//! }
//! ```
//! This generates:
//! ```rust
//! #[unsafe(link_section = ".progmem.data")]
//! static FOO_PROGMEM: [u8; 4] = [1, 2, 3, 4];
//! const FOO: progmem::ProgmemRef<[u8; 4]> = unsafe { progmem::ProgmemRef::new(&raw const FOO_PROGMEM) };
//! ```

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

/// Implements the `progmem` macro.
/// 
/// - Modifies the static variable to place it in the `.progmem.data` section.
/// - Creates a `ProgmemRef` constant to reference the variable safely.
pub(crate) fn progmem_impl(_args: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as syn::ItemStatic);

    let name = input.ident.clone();
    let ty = (*input.ty).clone();

    let new_name = format_ident!("{name}_PROGMEM");
    input.ident = new_name.clone();

    quote!(
        #[unsafe(link_section = ".progmem.data")]
        #input
        const #name: progmem::ProgmemRef<#ty> = unsafe {progmem::ProgmemRef::<#ty>::new(&raw const #new_name)};
    )
    .into()
}
