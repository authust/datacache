use proc_macro::TokenStream;
use storage::StorageArgs;
use syn::{parse_macro_input, DeriveInput};

mod attr;
mod data;
mod storage;

#[proc_macro_derive(DataMarker, attributes(datacache))]
pub fn derive_marker(input: TokenStream) -> TokenStream {
    data::derive(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}
#[proc_macro]
pub fn storage(input: TokenStream) -> TokenStream {
    storage::storage_expand(parse_macro_input!(input as StorageArgs))
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}
