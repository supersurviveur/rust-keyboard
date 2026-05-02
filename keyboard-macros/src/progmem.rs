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
use quote::ToTokens;
use syn::{ItemConst, parse_macro_input, parse_quote, parse_quote_spanned, spanned::Spanned};

/// Implements the `progmem` macro.
///
/// - Modifies the static variable to place it in the `.progmem.data` section.
/// - Creates a `ProgmemRef` constant to reference the variable safely.
pub(crate) fn progmem_impl(_args: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::ItemStatic);
    let expr = input.expr.clone();
    let ty = (*input.ty).clone();

    ItemConst {
        attrs: input.attrs,
        vis: input.vis,
        const_token: syn::token::Const {
            span: input.static_token.span(),
        },
        ident: input.ident,
        generics: syn::Generics {
            ..Default::default()
        },
        colon_token: input.colon_token,
        ty: parse_quote!(progmem::ProgmemRef::<#ty>),
        eq_token: input.eq_token,
        expr: parse_quote_spanned!(
            expr.span() =>
            unsafe {
                #[unsafe(link_section = ".progmem.data")]
                static PROGMEM_STORAGE:#ty = #expr;

                progmem::ProgmemRef::<#ty>::new(&raw const PROGMEM_STORAGE)
            }



        ),
        semi_token: input.semi_token,
    }
    .into_token_stream()
    .into()
}
