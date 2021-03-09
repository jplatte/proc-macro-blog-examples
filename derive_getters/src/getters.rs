use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DataStruct, DeriveInput, Fields, Ident, Lit, Meta, NestedMeta};

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
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics #st_name #ty_generics #where_clause {
            #(#getters)*
        }
    }
}

fn get_name_attr(attr: &Attribute) -> syn::Result<Option<Ident>> {
    let meta = attr.parse_meta()?;
    let meta_list = match meta {
        Meta::List(list) => list,
        _ => return Err(syn::Error::new_spanned(meta, "expected a list-style attribute")),
    };

    let nested = match meta_list.nested.len() {
        // `#[getter()]` without any arguments is a no-op
        0 => return Ok(None),
        1 => &meta_list.nested[0],
        _ => {
            return Err(syn::Error::new_spanned(
                meta_list.nested,
                "currently only a single getter attribute is supported",
            ));
        }
    };

    let name_value = match nested {
        NestedMeta::Meta(Meta::NameValue(nv)) => nv,
        _ => return Err(syn::Error::new_spanned(nested, "expected `name = \"<value>\"`")),
    };

    if !name_value.path.is_ident("name") {
        return Err(syn::Error::new_spanned(
            &name_value.path,
            "unsupported getter attribute, expected `name`",
        ));
    }

    match &name_value.lit {
        Lit::Str(s) => syn::parse_str(&s.value()).map_err(|e| syn::Error::new_spanned(s, e)),
        lit => Err(syn::Error::new_spanned(lit, "expected string literal")),
    }
}
