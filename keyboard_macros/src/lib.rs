//! # Keyboard Macros
//! 
//! This crate provides a collection of procedural macros for use in embedded systems programming,
//! particularly for keyboard firmware development. These macros simplify common tasks such as
//! managing pins, configuring constraints, and working with images and fonts.
//! 
//! ## Provided Macros
//! 
//! - **`#[progmem]`**: Places static variables in program memory (`.progmem.data` section).
//! - **`pins!`**: Generates constants for hardware pins.
//! - **`image_dimension!`**: Extracts dimensions and byte count from an image.
//! - **`include_font_plate!`**: Includes a font plate for rendering text.
//! - **`include_image!`**: Embeds an image into the firmware.
//! - **`include_animation!`**: Embeds an animation into the firmware.
//! - **`#[config_constraints]`**: Enforces compile-time constraints on generics.
//! - **`#[entry]`**: Marks the entry point of the firmware.
//! - **`#[key_alias]`**: Creates aliases for constants or structs.

mod constraints;
mod entry;
mod image;
mod key_alias;
mod pins;
mod progmem;

use proc_macro::TokenStream;
use quote::quote;

/// Places static variables in program memory.
#[proc_macro_attribute]
pub fn progmem(args: TokenStream, item: TokenStream) -> TokenStream {
    progmem::progmem_impl(args, item)
}

/// Generates constants for hardware pins.
#[proc_macro]
pub fn pins(item: TokenStream) -> TokenStream {
    pins::pins_impl(item)
}

/// Extracts dimensions and byte count from an image.
#[proc_macro]
pub fn image_dimension(input: TokenStream) -> TokenStream {
    let (width, height, _, byte_count, _) = image::parse_image(input).unwrap();

    let output = quote! {
        (#width,#height,#byte_count)
    };

    output.into()
}

#[proc_macro]
pub fn include_font_plate(input: TokenStream) -> TokenStream {
    image::include_font_plate_impl(input)
}

#[proc_macro]
pub fn include_image(input: TokenStream) -> TokenStream {
    image::include_image_impl(input)
}

#[proc_macro]
pub fn include_animation(input: TokenStream) -> TokenStream {
    image::include_animation_impl(input)
}

/// Enforces compile-time constraints on generics.
#[proc_macro_attribute]
pub fn config_constraints(args: TokenStream, item: TokenStream) -> TokenStream {
    constraints::config_constraints_impl(args, item)
}

/// Marks the entry point of the firmware.
#[proc_macro_attribute]
pub fn entry(args: TokenStream, item: TokenStream) -> TokenStream {
    entry::entry_impl(args, item)
}

/// Creates aliases for keys.
#[proc_macro_attribute]
pub fn key_alias(args: TokenStream, item: TokenStream) -> TokenStream {
    key_alias::key_alias_impl(args, item)
}
