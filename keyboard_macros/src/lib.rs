mod config;
mod constraints;
mod image;
mod keymap;
mod pins;
mod progmem;

use keymap::KeyboardDefinition;
use keymap::Keymap;
use proc_macro::Span;
use proc_macro::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use std::collections::HashSet;
use std::fs;
use syn::Expr;
use syn::ExprLit;
use syn::LitInt;
use syn::{ItemFn, parse_macro_input};

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
pub fn qmk_callback(args: TokenStream, item: TokenStream) -> TokenStream {
    qmk_callback_impl(args, item)
}

pub(crate) fn qmk_callback_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let ItemFn {
        mut sig,
        block,
        attrs,
        ..
    } = input;

    sig.ident = format_ident!("{}_rs", sig.ident);

    let statements = block.stmts;

    quote!(
        #(#attrs)*
        #[unsafe(no_mangle)]
        pub extern "C" #sig {
            #(#statements)*
        }
    )
    .into()
}

/// # Keymap
///
/// This macro is used to define the keymap. Use as follows:
/// ```rust
/// use qmk_macro::keymap;
///
/// keymap! {
///     "sofle/rev1",
///     {
///         KC_NO, KC_NO, KC_NO, // ...
///     }
/// }
/// ```
#[proc_macro]
pub fn keymap(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let keymap = parse_macro_input!(input as Keymap);
    let keyboard_definition = fs::read_to_string(format!(
        "../keyboards/{}/keyboard.json",
        keymap.keeb
    ))
    .unwrap_or_else(|_| {
        fs::read_to_string(format!("keyboards/{}/info.json", keymap.keeb))
            .unwrap_or_else(|_| panic!("Failed to read keyboard definition for {}", keymap.keeb))
    });

    let keyboard_definition: KeyboardDefinition = serde_json::from_str(&keyboard_definition)
        .unwrap_or_else(|_| panic!("Failed to parse keyboard definition for {}", keymap.keeb));

    let matrix_map = keyboard_definition
        .layouts
        .get("LAYOUT")
        .unwrap_or_else(|| {
            panic!(
                "Failed to find layout LAYOUT in keyboard definition for {}",
                keymap.keeb
            )
        });

    // find the number of unique matrix row values, ie matrix_map.layout[0].matrix[0]

    let matrix_rows = matrix_map
        .layout
        .iter()
        .map(|l| l.matrix[0])
        .collect::<HashSet<_>>()
        .len();

    let matrix_cols = matrix_map
        .layout
        .iter()
        .map(|l| l.matrix[1])
        .collect::<HashSet<_>>()
        .len();

    let mut layers = vec![];

    let num_layers = keymap.layers.len();

    for (x, layer) in keymap.layers.into_iter().enumerate() {
        let mut key_idents = vec![
            vec![
                Expr::Lit(ExprLit {
                    attrs: vec![],
                    lit: syn::Lit::Int(LitInt::new("0u16", Span::call_site().into())),
                });
                matrix_cols
            ];
            matrix_rows
        ];

        for (i, key) in layer.keys.into_iter().enumerate() {
            let Some(matrix_mapping) = matrix_map.layout.get(i).map(|m| m.matrix) else {
                panic!(
                    "Too many keys for {} in layer {x} (at key {i}, or '{}')",
                    keymap.keeb,
                    key.to_token_stream()
                );
            };
            let col = matrix_mapping[1] as usize;
            let row = matrix_mapping[0] as usize;
            // key_idents[row][col] = key;
            let Some(ident) = key_idents.get_mut(row).and_then(|r| r.get_mut(col)) else {
                panic!(
                    "Too many keys for {} in layer {x} (at key {i}, or '{}')",
                    keymap.keeb,
                    key.to_token_stream()
                );
            };

            *ident = key;
        }

        layers.push(key_idents);
    }

    let layers_tokens = layers
        .iter()
        .map(|layer| {
            let layer_tokens = layer
                .iter()
                .map(|row| {
                    let row_tokens = row.chunks(matrix_rows).map(|keys| {
                        let key_tokens = keys.iter().map(|key| {
                            if key.to_token_stream().to_string().starts_with("CS_") {
                                quote! {
                                    #key
                                }
                            } else {
                                quote! {
                                    ::qmk::key!(#key)
                                }
                            }
                        });

                        quote! {
                            #(#key_tokens),*
                        }
                    });
                    quote! {
                        [
                            #(#row_tokens),*
                        ]
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                [
                    #(#layer_tokens),*
                ]
            }
        })
        .collect::<Vec<_>>();

    let layers = quote! {
        [
            #(#layers_tokens),*
        ]
    };

    let output = quote! {
        #[unsafe(no_mangle)]
        #[allow(non_upper_case_globals)]
        #[unsafe(link_section = ".progmem.data")]
        static keymaps: [[[u16; #matrix_cols]; #matrix_rows]; #num_layers] = #layers;
    };

    output.into()
}
