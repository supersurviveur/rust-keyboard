use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned};

pub fn user_config_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut t = parse_macro_input!(item as syn::ItemImpl);

    // Tell to the user to remove type definition if they have one
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
        type KeymapType = qmk::keymap::Keymap<{ Self::LAYER_COUNT }>;
    }
    .into();
    t.items.push(parse_macro_input!(keymap as syn::ImplItem));
    quote! {#t}.into()
}
