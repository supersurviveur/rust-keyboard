//! # Pins Macro Implementation
//!
//! This file defines a procedural macro for generating constants representing hardware pins.
//! The macro takes a list of pins as input, where each pin is represented by a port (character) and a number.
//!
//! ## Example
//! ```rust
//! pins!(A0, B1, C2);
//! ```
//! This will generate constants like:
//! ```rust
//! pub const A0: Pin = Pin(...);
//! pub const B1: Pin = Pin(...);
//! pub const C2: Pin = Pin(...);
//! ```

use proc_macro::TokenStream;
use quote::ToTokens;
use quote::{format_ident, quote};
use syn::Token;
use syn::parse::Parse;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;

pub(crate) fn pins_impl(input: TokenStream) -> proc_macro::TokenStream {
    let pins = parse_macro_input!(input as Pins).0;
    let pins_names: Vec<_> = pins
        .clone()
        .iter()
        .map(|pin| format_ident!("{}{}", pin.0.to_ascii_uppercase(), pin.1))
        .collect();

    quote! {
        #(pub const #pins_names: Pin = Pin(#pins);)*
    }
    .into()
}

struct Pins(Vec<Pin>);

#[derive(Clone)]
struct Pin(char, u8);

impl Parse for Pin {
    //! Parses a single pin from the input stream.
    //! A pin is represented as a character (port) followed by a number (pin index).
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pin_ident = input.parse::<syn::Ident>()?;
        let binding = pin_ident.to_string();
        let mut pin = binding.chars();
        let c = pin.next().unwrap();
        let val = pin
            .collect::<String>()
            .parse()
            .map_err(|err| syn::Error::new(pin_ident.span(), format!("Invalid pin: {}", err)))?;
        Ok(Self(c, val))
    }
}
impl Parse for Pins {
    //! Parses a list of pins separated by commas.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pins = Punctuated::<Pin, Token![,]>::parse_terminated(input)?;
        Ok(Pins(pins.into_iter().collect()))
    }
}

impl ToTokens for Pin {
    //! Converts a `Pin` into its token representation.
    //! The token representation encodes the pin's port and index into a constant value.
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let port = format_ident!("PIN{}_ADDRESS", self.0.to_ascii_uppercase());
        let pin = self.1;
        tokens.extend(quote! {
            ((#port << crate::pins::PORT_SHIFTER) | #pin)
        })
    }
}
