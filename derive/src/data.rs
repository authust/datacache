use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Error, Field, Ident};

use crate::attr::{field_attr, filter_attributes, FieldAttr};

#[derive(Clone)]
struct QueryableField<'a> {
    idx: usize,
    field: Field,
    _data: FieldAttr,
    struct_ident: &'a Ident,
}

impl<'a> ToTokens for QueryableField<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attributes = filter_attributes(self.field.attrs.iter());
        let vis = &self.field.vis;
        let ident = match self.field.ident.clone() {
            Some(ident) => ident,
            None => Ident::new(&format!("f{}", self.idx), Span::call_site()),
        };
        let colon = self.field.colon_token;
        let ty = &self.field.ty;
        let struct_ident = self.struct_ident;
        quote!(#(#attributes)* #vis #ident #colon std::collections::HashMap<#ty, std::sync::Arc<#struct_ident>>)
            .to_tokens(tokens);
    }
}

#[repr(transparent)]
struct EnumField<'a>(QueryableField<'a>);

impl<'a> ToTokens for EnumField<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = match self.0.field.ident.clone() {
            Some(ident) => ident,
            None => {
                let idx = self.0.idx;
                return quote!(#idx(self.#idx.clone())).to_tokens(tokens);
            }
        };
        let ty = &self.0.field.ty;
        quote!(#ident(#ty)).to_tokens(tokens)
    }
}
#[repr(transparent)]
struct EnumCreateField<'a>(QueryableField<'a>);

impl<'a> ToTokens for EnumCreateField<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = match self.0.field.ident.clone() {
            Some(ident) => ident,
            None => Ident::new(&format!("F{}", self.0.idx), Span::call_site()),
        };
        quote!(#ident(self.#ident.clone())).to_tokens(tokens)
    }
}

pub fn derive(input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let data = match input.data {
        Data::Struct(data) => data,
        _ => {
            return Err(Error::new(
                Span::call_site(),
                "enums/unions are not supported",
            ))
        }
    };
    let mut fields = Vec::new();
    let mut f_idx = 0;
    for field in data.fields.into_iter() {
        let attr = field_attr(&field)?;
        f_idx += 1;
        if attr.queryable {
            fields.push(QueryableField {
                idx: f_idx,
                field,
                _data: attr,
                struct_ident: &input.ident,
            });
        }
    }

    let vis = input.vis;
    let ident = &input.ident;

    let enum_fields: Vec<EnumField> = fields.clone().into_iter().map(EnumField).collect();
    let query_ident = new_ident(&input.ident, "Query");
    let query_enum = quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[allow(non_camel_case_types)]
        #vis enum #query_ident {
            #(#enum_fields,)*
        }
    };
    let fields: Vec<EnumCreateField> = fields.into_iter().map(EnumCreateField).collect();

    let out = quote! {
        #query_enum

        impl datacache::DataMarker for #ident {
            type Query = #query_ident;
            fn create_queries(&self) -> Vec<Self::Query> {
                vec![#(#query_ident::#fields),*]
            }
        }
    };
    Ok(out)
}

fn new_ident(ident: &Ident, s: &'static str) -> Ident {
    Ident::new(&format!("{}{s}", ident), Span::call_site())
}
