//! This module defines types and structures for working with HID (Human Interface Device) descriptors.
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, Ident, LitInt, Token, parse::Parse, parse_macro_input, punctuated::Punctuated};

/// Represents the different types of HID items.
#[derive(Debug, Clone)]
pub enum HidItemType {
    /// Represents a usage page in the HID descriptor.
    UsagePage,
    /// Represents a specific usage within a usage page.
    Usage,
    /// Represents the start of a collection in the HID descriptor.
    Collection,
    /// Represents a feature report in the HID descriptor.
    Feature,
    /// Represents the minimum usage value in a range.
    UsageMinimum,
    /// Represents the maximum usage value in a range.
    UsageMaximum,
    /// Represents the minimum logical value for a control.
    LogicalMinimum,
    /// Represents the maximum logical value for a control.
    LogicalMaximum,
    /// Represents the minimum physical value for a control.
    PhysicalMinimum,
    /// Represents the maximum physical value for a control.
    PhysicalMaximum,
    /// Represents the size of a report field in bits.
    ReportSize,
    /// Represents the number of report fields.
    ReportCount,
    /// Pushes the current state onto the stack.
    Push,
    /// Pops the current state from the stack.
    Pop,
    /// Represents an input report in the HID descriptor.
    Input,
    /// Represents an output report in the HID descriptor.
    Output,
    /// Represents the end of a collection in the HID descriptor.
    EndCollection,
}

/// Represents an individual HID item in the descriptor.
#[derive(Clone)]
pub struct HidItem {
    /// The type of the HID item.
    r#type: HidItemType,
    /// An optional expression associated with the HID item.
    expr: Option<Expr>,
    /// The size of the HID item in bits.
    size: u8,
}

impl Parse for HidItem {
    /// Parses an HID item from a token stream.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let hid_type = input.parse::<Ident>()?;
        let hid = match hid_type.to_string().as_str() {
            "usage_page" => HidItemType::UsagePage,
            "usage" => HidItemType::Usage,
            "collection" => HidItemType::Collection,
            "feature" => HidItemType::Feature,
            "usage_maximum" => HidItemType::UsageMaximum,
            "usage_minimum" => HidItemType::UsageMinimum,
            "logical_maximum" => HidItemType::LogicalMaximum,
            "logical_minimum" => HidItemType::LogicalMinimum,
            "physical_maximum" => HidItemType::PhysicalMaximum,
            "physical_minimum" => HidItemType::PhysicalMinimum,
            "report_size" => HidItemType::ReportSize,
            "report_count" => HidItemType::ReportCount,
            "push" => HidItemType::Push,
            "pop" => HidItemType::Pop,
            "input" => HidItemType::Input,
            "output" => HidItemType::Output,
            "end_collection" => HidItemType::EndCollection,
            _ => return Err(syn::Error::new(hid_type.span(), "HID type is incorrect !")),
        };

        Ok(match hid {
            HidItemType::EndCollection => Self {
                r#type: hid,
                expr: None,
                size: 0,
            },
            _ => {
                let mut expr = None;
                let mut size = 0;
                if input.parse::<Option<Token![:]>>()?.is_some() {
                    size = 8;
                    expr = Some(input.parse::<Expr>()?);
                    if input.parse::<Option<Token![:]>>()?.is_some() {
                        size = input.parse::<LitInt>()?.base10_parse()?;
                    }
                }
                Self {
                    r#type: hid,
                    expr,
                    size,
                }
            }
        })
    }
}

impl ToTokens for HidItem {
    /// Converts the HID item into a token stream.
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let size_flag = match self.size {
            0 => 0,
            8 => 1,
            16 => 2,
            32 => 3,
            _ => panic!("Taille non supportée"),
        };

        // Constants defining the types of HID items.
        const MAIN: u8 = 0 << 2;
        const GLOBAL: u8 = 1 << 2;
        const LOCAL: u8 = 2 << 2;

        // Constants defining the tags for various HID items.
        const USAGE_PAGE: u8 = 0x00;
        const USAGE: u8 = 0x00;
        const COLLECTION: u8 = 0xA0;
        const FEATURE: u8 = 0xB0;
        const END_COLLECTION: u8 = 0xC0;
        const USAGE_MINIMUM: u8 = 0x10;
        const USAGE_MAXIMUM: u8 = 0x20;
        const LOGICAL_MINIMUM: u8 = 0x10;
        const LOGICAL_MAXIMUM: u8 = 0x20;
        const PHYSICAL_MINIMUM: u8 = 0x30;
        const PHYSICAL_MAXIMUM: u8 = 0x40;
        const REPORT_SIZE: u8 = 0x70;
        const REPORT_COUNT: u8 = 0x90;
        const PUSH: u8 = 0xA0;
        const POP: u8 = 0xB0;
        const INPUT: u8 = 0x80;
        const OUTPUT: u8 = 0x90;

        let (item_type, tag) = match self.r#type {
            HidItemType::UsagePage => (GLOBAL, USAGE_PAGE),
            HidItemType::Usage => (LOCAL, USAGE),
            HidItemType::Collection => (MAIN, COLLECTION),
            HidItemType::Feature => (MAIN, FEATURE),
            HidItemType::UsageMinimum => (LOCAL, USAGE_MINIMUM),
            HidItemType::UsageMaximum => (LOCAL, USAGE_MAXIMUM),
            HidItemType::LogicalMinimum => (GLOBAL, LOGICAL_MINIMUM),
            HidItemType::LogicalMaximum => (GLOBAL, LOGICAL_MAXIMUM),
            HidItemType::PhysicalMinimum => (GLOBAL, PHYSICAL_MINIMUM),
            HidItemType::PhysicalMaximum => (GLOBAL, PHYSICAL_MAXIMUM),
            HidItemType::ReportSize => (GLOBAL, REPORT_SIZE),
            HidItemType::ReportCount => (GLOBAL, REPORT_COUNT),
            HidItemType::Push => (GLOBAL, PUSH),
            HidItemType::Pop => (GLOBAL, POP),
            HidItemType::Input => (MAIN, INPUT),
            HidItemType::Output => (MAIN, OUTPUT),
            HidItemType::EndCollection => (MAIN, END_COLLECTION),
        };

        let header = item_type | tag | size_flag;
        tokens.extend(quote! { #header, });

        tokens.extend(match (self.size, &self.expr) {
            (0, None) => quote! {},
            (8, Some(expr)) => quote!(#expr, ),
            (16, Some(expr)) => {
                quote!(#expr as u8, ((#expr) as u16 >> 8) as u8, )
            }
            (32, Some(expr)) => {
                quote!(#expr as u8, ((#expr) as u32 >> 8) as u8, ((#expr) as u32 >> 16) as u8, ((#expr) as u32 >> 24) as u8, )
            }
            _ => unreachable!(),
        });
    }
}

/// Represents a collection of HID items.
#[derive(Clone)]
struct HidItems {
    /// A vector containing the individual HID items.
    items: Vec<HidItem>,
}

impl Parse for HidItems {
    /// Parses a collection of HID items from a token stream.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let items = Punctuated::<HidItem, Token![,]>::parse_terminated(input)?
            .into_iter()
            .collect();
        Ok(Self { items })
    }
}

impl ToTokens for HidItems {
    /// Converts the collection of HID items into a token stream.
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let inner = &self.items;
        tokens.extend(quote! {[#(#inner)*]});
    }
}

/// Generates an HID descriptor from a token stream.
pub fn hid_descriptor_impl(input: TokenStream) -> TokenStream {
    let items = parse_macro_input!(input as HidItems);
    quote! {
        #items
    }
    .into()
}
