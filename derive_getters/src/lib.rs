use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod getters;
use getters::expand_getters;

#[proc_macro_derive(Getters, attributes(getter))]
pub fn getters(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_getters(input).unwrap_or_else(syn::Error::into_compile_error).into()
}
