//! # Key Alias Macro Implementation
//!
//! This file defines a procedural macro for creating aliases for constants or structs.
//! The macro can be used to define alternative names (aliases) for keys or types, improving code readability and flexibility.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    Ident, ItemConst, ItemStruct, Token, parse::Parse, parse_macro_input, punctuated::Punctuated,
    spanned::Spanned,
};

struct Aliases(Vec<Ident>);

impl Parse for Aliases {
    //! Parses a list of identifiers (aliases) separated by commas.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(
            Punctuated::<Ident, Token![,]>::parse_terminated(input)?
                .into_iter()
                .collect(),
        ))
    }
}

pub fn key_alias_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    //! Implements the `key_alias` macro.
    let aliases = parse_macro_input!(args as Aliases).0;

    if let Ok(const_item) = syn::parse::<ItemConst>(item.clone()) {
        let base_ident = format_ident!("_BASE_{}", const_item.ident.clone());
        let old_base_ident = const_item.ident.clone();

        let base_value = match &*const_item.expr {
            syn::Expr::Reference(expr_ref) => expr_ref.expr.clone(),
            _ => return syn::Error::new(
                const_item.expr.span(),
                "key_alias macro must be used on a constant containing a value like &Key(value)",
            )
            .into_compile_error()
            .into(),
        };

        let mut res = quote! {};
        for alias in aliases {
            let doc = format!("Alias for [{}]", old_base_ident);
            res.extend(quote! {
                #[doc = #doc]
                pub const #alias: &Key = &#base_ident;
            });
        }

        quote! {
            const #base_ident: Key = #base_value;
            pub const #old_base_ident: &Key = &#base_ident;
            #res
        }
    } else if let Ok(struct_item) = syn::parse::<ItemStruct>(item) {
        let base_ident = struct_item.ident.clone();

        let mut res = quote! {};
        for alias in aliases {
            let doc = format!("Alias for [{}]", base_ident);
            res.extend(quote! {
                #[doc = #doc]
                pub type #alias = #base_ident;
            });
        }

        quote! {
            #struct_item
            #res
        }
    } else {
        syn::Error::new(
            Span::call_site(),
            "key_alias macro must be used on a struct or a const.",
        )
        .into_compile_error()
    }
    .into()
}
