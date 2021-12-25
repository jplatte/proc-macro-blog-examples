use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    Data, DataStruct, DeriveInput, Fields, Ident, LitStr, Token, Visibility,
};

pub fn expand_getters(input: DeriveInput) -> syn::Result<TokenStream> {
    let fields = match input.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };

    let getters = fields
        .into_iter()
        .map(|f| {
            let meta: GetterMeta = f
                .attrs
                .iter()
                .filter(|attr| attr.path.is_ident("getter"))
                .try_fold(GetterMeta::default(), |meta, attr| {
                    let list: Punctuated<GetterMeta, Token![,]> =
                        attr.parse_args_with(Punctuated::parse_terminated)?;

                    list.into_iter().try_fold(meta, GetterMeta::merge)
                })?;

            let visibility = meta.vis.unwrap_or_else(|| parse_quote! { pub });
            let method_name = meta.name.unwrap_or_else(|| f.ident.clone().expect("a named field"));
            let field_name = f.ident;
            let field_ty = f.ty;

            let deprecation_note = meta.deprecated_name_syntax.then(|| {
                quote_spanned! {method_name.span()=>
                    #[deprecated = "Using a string literal as a name attribute is deprecated. \
                        Use an identifier instead (remove the quotes)."]
                    fn name_literal() {}
                    name_literal();
                }
            });

            Ok(quote! {
                #visibility fn #method_name(&self) -> &#field_ty {
                    #deprecation_note
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

#[derive(Default)]
struct GetterMeta {
    name: Option<Ident>,
    vis: Option<Visibility>,

    deprecated_name_syntax: bool,
}

impl GetterMeta {
    fn merge(self, other: GetterMeta) -> syn::Result<Self> {
        fn either<T: ToTokens>(a: Option<T>, b: Option<T>) -> syn::Result<Option<T>> {
            match (a, b) {
                (None, None) => Ok(None),
                (Some(val), None) | (None, Some(val)) => Ok(Some(val)),
                (Some(a), Some(b)) => {
                    let mut error = syn::Error::new_spanned(a, "redundant attribute argument");
                    error.combine(syn::Error::new_spanned(b, "note: first one here"));
                    Err(error)
                }
            }
        }

        Ok(Self {
            name: either(self.name, other.name)?,
            vis: either(self.vis, other.vis)?,
            deprecated_name_syntax: self.deprecated_name_syntax || other.deprecated_name_syntax,
        })
    }
}

impl Parse for GetterMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::name) {
            let _: kw::name = input.parse()?;
            let _: Token![=] = input.parse()?;

            let lookahead = input.lookahead1();
            let (name, deprecated_name_syntax) = if lookahead.peek(Ident) {
                (input.parse()?, false)
            } else if lookahead.peek(LitStr) {
                let s: LitStr = input.parse()?;
                let span = s.span(); // We need this span for error reporting

                let mut name: Ident =
                    syn::parse_str(&s.value()).map_err(|e| syn::Error::new_spanned(s, e))?;
                name.set_span(span);

                (name, true)
            } else {
                return Err(lookahead.error());
            };

            Ok(Self { name: Some(name), vis: None, deprecated_name_syntax })
        } else if lookahead.peek(kw::vis) {
            let _: kw::vis = input.parse()?;
            let _: Token![=] = input.parse()?;
            let vis = input.parse()?;

            Ok(Self { name: None, vis: Some(vis), deprecated_name_syntax: false })
        } else {
            Err(lookahead.error())
        }
    }
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(name);
    custom_keyword!(vis);
}
