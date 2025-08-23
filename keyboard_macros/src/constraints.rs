//! # Config Constraints Macro Implementation
//!
//! This module provides the implementation for the `config_constraints` procedural macro.
//! The macro is used to enforce compile-time constraints on generics in structs, traits, or functions.
//! It ensures that certain conditions are met based on a configurable prefix (e.g., `User`).
//!
//! ## Usage
//! The `config_constraints` macro can be applied to:
//! - Structs
//! - Traits
//! - Trait constants
//! - Trait functions
//! - Impl blocks
//!
//! ## Example
//! ```rust
//! config_constraints!(User => struct MyStruct<T>);
//! ```
//! This will add compile-time constraints to `MyStruct` based on the `User` prefix, such as:
//! - `User::LAYER_COUNT`
//! - `User::ROWS_PER_HAND`
//! - `User::MATRIX_ROWS * User::MATRIX_COLUMNS`

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, TokenStreamExt, quote};
use syn::{
    AttrStyle, Expr, ImplItemFn, ItemImpl, ItemStruct, ItemTrait, TraitItemConst, TraitItemFn,
    WhereClause, parse::Parse, parse_macro_input, parse_quote,
};

/// Represents the arguments for the `config_constraints` macro.
///
/// The arguments include a `prefix` expression, which defaults to `User` if not provided.
struct Args {
    /// The prefix expression used to qualify the constraints.
    prefix: Expr,
}

impl Parse for Args {
    /// Parses the arguments for the `config_constraints` macro.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let prefix = input.parse().unwrap_or(parse_quote!(User));
        Ok(Self { prefix })
    }
}

/// Custom to token methods, otherwise generics aren't produced, see issue syn#782
fn const_item_with_generics(item: TraitItemConst, tokens: &mut TokenStream2) {
    tokens.append_all(
        item.attrs
            .iter()
            .filter(|attr| attr.style == AttrStyle::Outer),
    );
    item.const_token.to_tokens(tokens);
    item.ident.to_tokens(tokens);
    item.colon_token.to_tokens(tokens);
    item.ty.to_tokens(tokens);
    if let Some((eq_token, default)) = &item.default {
        eq_token.to_tokens(tokens);
        default.to_tokens(tokens);
    }
    item.generics.to_tokens(tokens);
    item.generics.where_clause.to_tokens(tokens);
    item.semi_token.to_tokens(tokens);
}

/// Implements the `config_constraints` macro.
pub fn config_constraints_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let prefix = args.prefix;

    let constraints: &[TokenStream2] = &[
        quote! {
            #prefix::LAYER_COUNT
        },
        quote! {
            #prefix::ROWS_PER_HAND as usize
        },
        quote! {
            #prefix::MATRIX_ROWS as usize * #prefix::MATRIX_COLUMNS as usize
        },
        quote! {
            #prefix::FONT_SIZE
        },
        quote! {
            #prefix::CHAR_HEIGHT as usize
        },
        quote! {
            #prefix::CHAR_WIDTH as usize
        },
        quote! {
            #prefix::FONT_HEIGHT as usize
        },
        quote! {
            #prefix::FONT_WIDTH as usize
        },
    ];

    let apply_clauses = |where_clause: &mut WhereClause| {
        for constraint in constraints {
            let predicate = parse_quote! {
                [(); #constraint]:
            };
            where_clause.predicates.push(predicate);
        }
    };

    if let Ok(mut impl_trait) = syn::parse::<ItemImpl>(item.clone()) {
        let where_clause = impl_trait.generics.make_where_clause();

        apply_clauses(where_clause);

        quote! {#impl_trait}
    } else if let Ok(mut item_struct) = syn::parse::<ItemStruct>(item.clone()) {
        let where_clause = item_struct.generics.make_where_clause();

        apply_clauses(where_clause);

        quote! {#item_struct}
    } else if let Ok(mut item_trait) = syn::parse::<ItemTrait>(item.clone()) {
        let where_clause = item_trait.generics.make_where_clause();

        apply_clauses(where_clause);

        quote! {#item_trait}
    } else if let Ok(mut item_trait_const) = syn::parse::<TraitItemConst>(item.clone()) {
        let where_clause = item_trait_const.generics.make_where_clause();

        apply_clauses(where_clause);

        // Custom to token methods, otherwise generics aren't produced
        let mut stream = TokenStream2::new();
        const_item_with_generics(item_trait_const, &mut stream);
        stream
    } else if let Ok(mut item_trait_fn) = syn::parse::<TraitItemFn>(item.clone()) {
        let where_clause = item_trait_fn.sig.generics.make_where_clause();

        apply_clauses(where_clause);

        quote! {#item_trait_fn}
    } else if let Ok(mut item_trait_fn) = syn::parse::<ImplItemFn>(item.clone()) {
        let where_clause = item_trait_fn.sig.generics.make_where_clause();

        apply_clauses(where_clause);

        quote! {#item_trait_fn}
    } else {
        syn::Error::new(
            Span::call_site(),
            "config_constraints macro must be used on a struct, an impl, a trait, a const in a trait, a function in a trait or a function in an impl.",
        )
        .into_compile_error()

    }.into()
}
