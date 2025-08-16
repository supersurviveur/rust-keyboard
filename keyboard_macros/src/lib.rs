mod config;
mod constraints;
mod entry;
mod image;
mod pins;
mod progmem;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn progmem(args: TokenStream, item: TokenStream) -> TokenStream {
    progmem::progmem_impl(args, item)
}
#[proc_macro]
pub fn pins(item: TokenStream) -> TokenStream {
    pins::pins_impl(item)
}
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

#[proc_macro_attribute]
pub fn user_config(args: TokenStream, item: TokenStream) -> TokenStream {
    config::user_config_impl(args, item)
}

#[proc_macro_attribute]
pub fn config_constraints(args: TokenStream, item: TokenStream) -> TokenStream {
    constraints::config_constraints_impl(args, item)
}
#[proc_macro_attribute]
pub fn entry(args: TokenStream, item: TokenStream) -> TokenStream {
    entry::entry_impl(args, item)
}
