use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Data, DataStruct, DeriveInput, Fields, Ident, Token,
};

pub fn expand_getters(input: DeriveInput) -> syn::Result<TokenStream> {
    let fields = match input.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };

    let getters = fields
        .into_iter()
        .map(|f| {
            let attrs: Vec<_> =
                f.attrs.iter().filter(|attr| attr.path.is_ident("getter")).collect();

            let name_from_attr = match attrs.len() {
                0 => None,
                1 => get_name_attr(attrs[0])?,
                _ => {
                    let mut error =
                        syn::Error::new_spanned(attrs[1], "redundant `getter(name)` attribute");
                    error.combine(syn::Error::new_spanned(attrs[0], "note: first one here"));
                    return Err(error);
                }
            };

            // if there is no `getter(name)` attribute use the field name like before
            let method_name =
                name_from_attr.unwrap_or_else(|| f.ident.clone().expect("a named field"));
            let field_name = f.ident;
            let field_ty = f.ty;

            Ok(quote! {
                pub fn #method_name(&self) -> &#field_ty {
                    &self.#field_name
                }
            })
        })
        .collect::<syn::Result<TokenStream>>()?;

    let st_name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #st_name #ty_generics #where_clause {
            #getters
        }
    })
}

fn get_name_attr(attr: &Attribute) -> syn::Result<Option<Ident>> {
    let meta: GetterMeta = attr.parse_args()?;
    Ok(meta.name)
}

struct GetterMeta {
    name: Option<Ident>,
}

impl Parse for GetterMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let arg_name: Ident = input.parse()?;
        if arg_name != "name" {
            return Err(syn::Error::new_spanned(
                arg_name,
                "unsupported getter attribute, expected `name`",
            ));
        }

        let _: Token![=] = input.parse()?;
        let name = input.parse()?;

        Ok(Self { name: Some(name) })
    }
}
