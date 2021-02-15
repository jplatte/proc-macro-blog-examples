use proc_macro2::TokenStream;
use syn::{Data, DataStruct, DeriveInput, Fields};

pub fn expand_getters(input: DeriveInput) -> TokenStream {
    let fields = match input.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };

    TokenStream::new()
}
