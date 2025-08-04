mod hid_descriptor;

use proc_macro::TokenStream;
use quote::quote;
use syn::Lit;
use syn::parse_macro_input;

#[proc_macro]
pub fn hid_descriptor(input: TokenStream) -> TokenStream {
    hid_descriptor::hid_descriptor_impl(input)
}

#[proc_macro]
pub fn literal_to_wchar_array(input: TokenStream) -> TokenStream {
    let mut res: Vec<i16> = Vec::new();
    let lit = parse_macro_input!(input as Lit);
    match lit {
        Lit::Str(str_lit) => {
            for c in str_lit.value().chars() {
                res.push(TryInto::<u16>::try_into(c).unwrap() as i16);
            }
            quote! {[#(#res),*]}
        }
        _ => {
            syn::Error::new(lit.span(), "Literal must be a string".to_string()).into_compile_error()
        }
    }
    .into()
}
