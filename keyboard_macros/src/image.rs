use std::fs;

use image::{ImageBuffer, Luma};
use proc_macro::{Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::{parse::Parse, parse_macro_input};

fn remove_non_alphanumeric(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9_]+").unwrap();
    re.replace_all(input, "").to_string()
}

fn to_format(img: ImageBuffer<Luma<u8>, Vec<u8>>, width: usize, height: usize) -> Vec<u8> {
    let mut output = Vec::new();

    let mut bit: u8 = 0;
    let mut byte: u8 = 0;
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x as u32, y as u32)[0];
            if pixel >= 127 {
                byte |= 1 << bit;
            }
            bit += 1;
            if bit == 8 {
                output.push(byte);
                byte = 0;
                bit = 0;
            }
        }
    }
    if bit != 0 {
        output.push(byte);
    }
    output
}

fn path_to_image(path: &str) -> (Vec<u8>, String, usize, usize) {
    let img = match image::open(path) {
        Ok(img) => img.to_luma8(),
        Err(e) => panic!("failed to open image {}: {}", path, e),
    };

    let width = img.width() as usize;
    let height = img.height() as usize;

    let bytes = to_format(img, width, height);

    let path = path
        .split('/')
        .next_back()
        .expect("failed to get last part of path");
    let split: Vec<_> = path.split('.').collect();
    let name = remove_non_alphanumeric(&split[0..split.len() - 1].join(".")).to_uppercase();

    (bytes, name, width, height)
}

struct ParsedArgs {
    path: String,
}

impl Parse for ParsedArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path: syn::LitStr = input.parse()?;
        let path = path.value();
        Ok(ParsedArgs { path })
    }
}

pub fn parse_image(
    input: TokenStream,
) -> Result<
    (
        u8,
        u8,
        proc_macro2::Ident,
        usize,
        Vec<proc_macro2::TokenStream>,
    ),
    syn::Error,
> {
    // parse the input into a comma separated list of arguments
    let parsed_args = syn::parse::<ParsedArgs>(input)?;
    // let parsed_args = parse_macro_input!(input as ParsedArgs);
    let (bytes, name, width, height) = path_to_image(&parsed_args.path);

    let width = width as u8;
    let height = height as u8;

    let byte_array = bytes.as_slice();
    let byte_count = byte_array.len();

    let name_ident = syn::Ident::new(&name, Span::call_site().into());

    let byte_tokens = bytes.iter().map(|b| quote! { #b }).collect::<Vec<_>>();
    Ok((width, height, name_ident, byte_count, byte_tokens))
}

pub fn include_font_plate_impl(input: TokenStream) -> TokenStream {
    let (_, _, _, _, byte_tokens) = parse_image(input).unwrap();

    let output = quote! {
        [#(#byte_tokens),*]
    };
    output.into()
}

pub fn include_image_impl(input: TokenStream) -> TokenStream {
    let (width, height, _, _, byte_tokens) = parse_image(input).unwrap();

    let output = quote! {
        ::include_image::QmkImage {
            width: #width,
            height: #height,
            bytes: [#(#byte_tokens),*],
        }
    };

    output.into()
}

pub fn include_animation_impl(input: TokenStream) -> TokenStream {
    // let input_path = parse_macro_input!(input as syn::LitStr).value();
    let parsed_args = parse_macro_input!(input as ParsedArgs);

    let files = match fs::read_dir(&parsed_args.path) {
        Ok(res) => res,
        Err(e) => panic!("failed to read animation directory: {}", e),
    };

    let name_ident = syn::Ident::new(
        &remove_non_alphanumeric(parsed_args.path.split("/").last().expect("invalid path"))
            .to_uppercase(),
        Span::call_site().into(),
    );

    let mut files: Vec<_> = files
        .into_iter()
        .filter_map(|f| f.ok())
        .filter_map(|f| {
            f.file_name()
                .to_str()
                .map(|name| (name.to_string(), f.path()))
        })
        .filter_map(|(name, path)| fs::read(&path).ok().map(|content| (name, content)))
        .collect();

    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut images_tokens = vec![];

    let mut all_lens = 0;

    for (name, _) in files.into_iter() {
        let (bytes, _name, width, height) =
            path_to_image(&format!("{}/{}", parsed_args.path, name));

        if all_lens == 0 {
            all_lens = bytes.len();
        } else if all_lens != bytes.len() {
            panic!("non-equal image sizes");
        }

        let width = width as u8;
        let height = height as u8;

        let byte_tokens = bytes.iter().map(|b| quote! { #b }).collect::<Vec<_>>();

        let byte_count = bytes.len();

        images_tokens.push(quote! {
            ::include_image::QmkImage::<#byte_count> {
                width: #width,
                height: #height,
                bytes: [#(#byte_tokens),*],
            }
        });
    }

    let images_tokens_len = images_tokens.len();

    // let images_tokens = images_tokens
    //     .iter()
    //     .fold(quote! {}, |acc, new| quote! {#acc #new});

    let output = quote! {
        pub const #name_ident: [::include_image::QmkImage<#all_lens>; #images_tokens_len] = [
            #(#images_tokens),*
        ];
    };

    output.into()
}
