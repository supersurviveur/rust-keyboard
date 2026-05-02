//! This module provides the implementation for the `user_config` procedural macro.
//!
//! The macro ensures that certain types are not manually defined by the user and automatically
//! adds required type definitions to the implementation block.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned};

/// Implements the `user_config` macro.
///
/// This function processes an implementation block, checks for prohibited type definitions,
/// and adds the required `KeymapType` definition if it is not already present.
pub fn user_config_impl(_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut t = parse_macro_input!(item as syn::ItemImpl);

    // Tell the user to remove type definitions if they have one
    const BLACK_LIST: [&str; 1] = ["KeymapType"];
    for item in &t.items {
        match item {
            syn::ImplItem::Type(item_type)
                if BLACK_LIST.contains(&item_type.ident.to_string().as_str()) =>
            {
                return syn::Error::new(
                    item_type.span(),
                    "This type is auto-implemented by the `user_config` macro, remove this definition.",
                ).into_compile_error().into();
            }
            _ => {}
        }
    }

    let keymap = quote! {
        type KeymapType = omk::keymap::Keymap<Self, { Self::LAYER_COUNT }>;
    }
    .into();
    t.items.push(parse_macro_input!(keymap as syn::ImplItem));
    quote! {#t}.into()
}
