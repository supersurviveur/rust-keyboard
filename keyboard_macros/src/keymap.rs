use std::collections::HashMap;

use serde::Deserialize;
use syn::{braced, parse::Parse, Expr, Token};

pub struct Keymap {
    pub keeb: String,
    pub layers: Vec<Layer>,
}

impl Parse for Keymap {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // comma seperated arguments
        let mut layers = Vec::new();
        let lit = input.parse::<syn::LitStr>()?;
        let keeb = lit.value();
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            while !input.is_empty() {
                let content;
                braced!(content in input);
                while !content.is_empty() {
                    let layer: Layer = content.parse()?;
                    layers.push(layer);
                    if content.peek(Token![,]) {
                        content.parse::<Token![,]>()?;
                    }
                }
                if !input.is_empty() {
                    input.parse::<Token![,]>()?;
                }
            }
        }

        Ok(Keymap { keeb, layers })
    }
}

pub struct Layer {
    pub keys: Vec<Expr>,
}

impl Parse for Layer {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut keys = Vec::new();
        while !input.is_empty() {
            let key: Expr = input.parse()?;
            keys.push(key);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Layer { keys })
    }
}

#[derive(Deserialize)]
pub struct KeyboardDefinition {
    pub layouts: HashMap<String, KeebDefLayout>,
}

#[derive(Deserialize)]
pub struct KeebDefLayout {
    pub layout: Vec<MatrixMapping>,
}

#[derive(Deserialize)]
pub struct MatrixMapping {
    pub matrix: [u8; 2],
}
