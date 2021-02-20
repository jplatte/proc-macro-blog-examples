use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Data, DataStruct, DeriveInput, Fields, GenericArgument, Path, PathArguments, Type, TypePath,
    TypeReference,
};

pub fn expand_getters(input: DeriveInput) -> TokenStream {
    let fields = match input.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };

    let getters = fields.into_iter().map(|f| {
        let field_name = f.ident;
        let (return_ty, body) = match f.ty {
            Type::Reference(r @ TypeReference { mutability: None, .. }) => {
                (quote! { #r }, quote! { self.#field_name })
            }
            Type::Path(TypePath { path, .. }) if path.is_ident("String") => {
                (quote! { &::core::primitive::str }, quote! { &self.#field_name })
            }
            Type::Path(ty @ TypePath { .. }) => match option_inner_type(&ty.path) {
                Some(Type::Path(TypePath { path, .. })) if path.is_ident("String") => (
                    quote! { ::std::option::Option<&::core::primitive::str> },
                    quote! { self.#field_name.as_deref() },
                ),
                Some(inner_ty) => (
                    quote! { ::std::option::Option<&#inner_ty> },
                    quote! { self.#field_name.as_ref() },
                ),
                None => (quote! { &#ty }, quote! { &self.#field_name }),
            },
            ty => (quote! { &#ty }, quote! { &self.#field_name }),
        };

        quote! {
            pub fn #field_name(&self) -> #return_ty {
                #body
            }
        }
    });

    let st_name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics #st_name #ty_generics #where_clause {
            #(#getters)*
        }
    }
}

fn option_inner_type(path: &Path) -> Option<&Type> {
    if path.leading_colon.is_some() {
        return None;
    }

    if path.segments.len() != 1 || path.segments[0].ident != "Option" {
        return None;
    }

    let ab = match &path.segments[0].arguments {
        PathArguments::AngleBracketed(ab) => ab,
        _ => return None,
    };

    if ab.args.len() != 1 {
        return None;
    }

    match &ab.args[0] {
        GenericArgument::Type(t) => Some(t),
        _ => None,
    }
}
