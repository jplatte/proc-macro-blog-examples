use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields};

pub fn expand_getters(input: DeriveInput) -> TokenStream {
    let fields = match input.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };

    let getters = fields.into_iter().map(|f| {
        let field_name = f.ident;
        let field_ty = f.ty;

        quote! {
            pub fn #field_name(&self) -> &#field_ty {
                &self.#field_name
            }
        }
    });

    let st_name = input.ident;

    quote! {
        #[automatically_derived]
        impl #st_name {
            #(#getters)*
        }
    }
}
