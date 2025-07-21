mod image;
mod keymap;
mod pins;
mod progmem;

use keymap::KeyboardDefinition;
use keymap::Keymap;
use proc_macro::Span;
use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use std::collections::HashSet;
use std::fs;
use syn::Expr;
use syn::ExprLit;
use syn::LitInt;
use syn::{parse_macro_input, ItemFn};

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

#[proc_macro]
pub fn hid_keyboard_descriptor(input: TokenStream) -> TokenStream {
    let max_keys = parse_macro_input!(input as LitInt);
    let max_keys_val: u8 = max_keys.base10_parse().unwrap();

    // Construire le descripteur sous forme de tableau d'octets
    let descriptor_bytes = build_descriptor(max_keys_val);

    // Générer le code Rust pour le tableau constant
    let expanded = quote! {
        [#(#descriptor_bytes),*]
    };

    expanded.into()
}

fn build_descriptor(max_keys: u8) -> Vec<u8> {
    let mut bytes = Vec::new();

    // Fonction utilitaire pour ajouter des items HID
    let mut push_item = |item_type: u8, tag: u8, size: u8, value: u32| {
        let size_flag = match size {
            0 => 0,
            8 => 1,
            16 => 2,
            32 => 3,
            _ => panic!("Taille non supportée"),
        };

        bytes.push(item_type | tag | size_flag);

        match size {
            0 => {}
            8 => bytes.push(value as u8),
            16 => {
                bytes.push(value as u8);
                bytes.push((value >> 8) as u8);
            }
            32 => {
                bytes.push(value as u8);
                bytes.push((value >> 8) as u8);
                bytes.push((value >> 16) as u8);
                bytes.push((value >> 24) as u8);
            }
            _ => unreachable!(),
        }
    };

    // Types et tags
    const MAIN: u8 = 0 << 2;
    const GLOBAL: u8 = 1 << 2;
    const LOCAL: u8 = 2 << 2;

    const USAGE_PAGE: u8 = 0x00;
    const USAGE: u8 = 0x00;
    const COLLECTION: u8 = 0xA0;
    const END_COLLECTION: u8 = 0xC0;
    const USAGE_MINIMUM: u8 = 0x10;
    const USAGE_MAXIMUM: u8 = 0x20;
    const LOGICAL_MINIMUM: u8 = 0x10;
    const LOGICAL_MAXIMUM: u8 = 0x20;
    const REPORT_SIZE: u8 = 0x70;
    const REPORT_COUNT: u8 = 0x90;
    const INPUT: u8 = 0x80;
    const OUTPUT: u8 = 0x90;

    // Flags
    const DATA: u8 = 0 << 0;
    const CONSTANT: u8 = 1 << 0;
    const ARRAY: u8 = 0 << 1;
    const VARIABLE: u8 = 1 << 1;
    const ABSOLUTE: u8 = 0 << 2;
    const NON_VOLATILE: u8 = 0 << 7;

    // Construction du descripteur
    push_item(GLOBAL, USAGE_PAGE, 8, 0x01);
    push_item(LOCAL, USAGE, 8, 0x06);
    push_item(MAIN, COLLECTION, 8, 0x01);

    push_item(GLOBAL, USAGE_PAGE, 8, 0x07);
    push_item(LOCAL, USAGE_MINIMUM, 8, 0xE0);
    push_item(LOCAL, USAGE_MAXIMUM, 8, 0xE7);
    push_item(GLOBAL, LOGICAL_MINIMUM, 8, 0x00);
    push_item(GLOBAL, LOGICAL_MAXIMUM, 8, 0x01);
    push_item(GLOBAL, REPORT_SIZE, 8, 0x01);
    push_item(GLOBAL, REPORT_COUNT, 8, 0x08);
    push_item(MAIN, INPUT, 8, (DATA | VARIABLE | ABSOLUTE) as u32);

    push_item(GLOBAL, REPORT_COUNT, 8, 0x01);
    push_item(GLOBAL, REPORT_SIZE, 8, 0x08);
    push_item(MAIN, INPUT, 8, CONSTANT as u32);

    push_item(GLOBAL, USAGE_PAGE, 8, 0x08);
    push_item(LOCAL, USAGE_MINIMUM, 8, 0x01);
    push_item(LOCAL, USAGE_MAXIMUM, 8, 0x05);
    push_item(GLOBAL, REPORT_COUNT, 8, 0x05);
    push_item(GLOBAL, REPORT_SIZE, 8, 0x01);
    push_item(
        MAIN,
        OUTPUT,
        8,
        (DATA | VARIABLE | ABSOLUTE | NON_VOLATILE) as u32,
    );

    push_item(GLOBAL, REPORT_COUNT, 8, 0x01);
    push_item(GLOBAL, REPORT_SIZE, 8, 0x03);
    push_item(MAIN, OUTPUT, 8, CONSTANT as u32);

    push_item(GLOBAL, LOGICAL_MINIMUM, 8, 0x00);
    push_item(GLOBAL, LOGICAL_MAXIMUM, 8, 0x65);
    push_item(GLOBAL, USAGE_PAGE, 8, 0x07);
    push_item(LOCAL, USAGE_MINIMUM, 8, 0x00);
    push_item(LOCAL, USAGE_MAXIMUM, 8, 0x65);
    push_item(GLOBAL, REPORT_COUNT, 8, max_keys as u32);
    push_item(GLOBAL, REPORT_SIZE, 8, 0x08);
    push_item(MAIN, INPUT, 8, (DATA | ARRAY | ABSOLUTE) as u32);

    push_item(MAIN, END_COLLECTION, 0, 0);

    bytes
}
