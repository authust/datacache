use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Colon, Comma},
    Error, Ident, Token, TypePath, Visibility,
};

mod kw {
    syn::custom_keyword!(fields);
    syn::custom_keyword!(unique);
    syn::custom_keyword!(id);
}

pub(crate) struct StorageArgs {
    visibility: Visibility,
    ident: Ident,
    executor_path: TypePath,
    data_path: TypePath,
    id_field: FieldTuple,
    #[allow(dead_code)]
    unique_fields: Vec<StorageField>,
    #[allow(dead_code)]
    query_fields: Vec<StorageField>,
}

pub(crate) struct StorageField(FieldTuple);
pub(crate) struct UniqueField(FieldTuple);

pub(crate) struct FieldTuple(Ident, TypePath);

impl Parse for StorageField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(FieldTuple::parse(input)?))
    }
}
impl Parse for UniqueField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(FieldTuple::parse(input)?))
    }
}

impl Parse for FieldTuple {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        input.parse::<Colon>()?;
        let path = input.parse()?;
        Ok(Self(ident, path))
    }
}
impl ToTokens for StorageField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.0 .0;
        let ty = &self.0 .1;
        quote!(#ident: #ty).to_tokens(tokens)
    }
}

impl Parse for StorageArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?;
        let ident: Ident = input.parse()?;

        let content;
        parenthesized!(content in input);
        let executor_path: TypePath = content.parse()?;
        content.parse::<Token![,]>()?;
        let data_path: TypePath = content.parse()?;

        input.parse::<Token![,]>()?;

        let id_field: FieldTuple = {
            input.parse::<kw::id>()?;
            let content;
            parenthesized!(content in input);
            content.parse()?
        };

        let unique_fields: Punctuated<StorageField, Comma> = {
            input.parse::<Token![,]>()?;

            input.parse::<kw::unique>()?;
            let content;
            parenthesized!(content in input);

            Punctuated::parse_terminated(&content)?
        };
        let query_fields: Punctuated<StorageField, Comma> = {
            input.parse::<Token![,]>()?;

            input.parse::<kw::fields>()?;
            let content;
            parenthesized!(content in input);

            Punctuated::parse_terminated(&content)?
        };
        Ok(Self {
            visibility: vis,
            ident,
            executor_path,
            data_path,
            id_field,
            unique_fields: unique_fields.into_iter().collect(),
            query_fields: query_fields.into_iter().collect(),
        })
    }
}

pub(crate) fn storage_expand(input: StorageArgs) -> Result<TokenStream, Error> {
    let StorageArgs {
        visibility: vis,
        ident,
        data_path,
        id_field: FieldTuple(id_field, _),
        executor_path,
        unique_fields: _,
        query_fields: _,
    } = input;
    let out = quote! {
        #vis struct #ident {
            executor: #executor_path,
            data: datacache::__internal::moka::future::Cache<<#executor_path as DataQueryExecutor<#data_path>>::Id, Data<#data_path>>,
            query_cache: datacache::__internal::moka::future::Cache<<#data_path as DataMarker>::Query, Data<#data_path>>,
            query: datacache::__internal::dashmap::DashMap<<#data_path as DataMarker>::Query, <#executor_path as DataQueryExecutor<#data_path>>::Id>,
        }

        impl #ident {
            pub fn new(executor: #executor_path) -> Self {
                Self {
                    executor,
                    data: datacache::__internal::moka::future::Cache::builder().build(),
                    query_cache: datacache::__internal::moka::future::Cache::builder().build(),
                    query: datacache::__internal::dashmap::DashMap::new(),
                }
            }

            fn find_id(&self, query: &<#data_path as DataMarker>::Query) -> Option<<#executor_path as DataQueryExecutor<#data_path>>::Id> {
                type Lol_Query = <#data_path as DataMarker>::Query;
                match query {
                    Lol_Query::#id_field(id) => Some(id.clone()),
                    other => self.query.get(other).map(|v| v.value().clone()),
                }
            }

            async fn insert_data(&self, data: Data<#data_path>) {
                for query in data.create_queries() {
                    self.query.insert(query, data.#id_field);
                }
                self.data.insert(data.#id_field, data).await;
            }
        }

        #[datacache::__internal::async_trait]
        impl DataStorage<#executor_path, #data_path> for #ident {
            async fn find_one(
                &self,
                query: <#data_path as DataMarker>::Query,
            ) -> Result<Data<#data_path>, Arc<<#executor_path as DataQueryExecutor<#data_path>>::Error>> {
                if let Some(id) = self.find_id(&query) {
                    if let Some(data) = self.data.get(&id) {
                        return Ok(data);
                    }
                }
                let fut = datacache::__internal::FutureExt::map(self
                    .executor
                    .find_one(query.clone()), |out| out.map(Data::new));
                let data = self.query_cache.try_get_with(query, fut).await?;
                self.insert_data(data.clone()).await;
                Ok(data)
            }
            async fn find_all(
                &self,
                query: <#data_path as DataMarker>::Query,
            ) -> Result<Vec<Data<#data_path>>, Arc<<#executor_path as DataQueryExecutor<#data_path>>::Error>>
            {
                let ids = self.executor.find_all_ids(query).await?;
                let mut values = Vec::new();
                for id in ids {
                    let data = self.find_one(<#data_path as DataMarker>::Query::#id_field(id)).await?;
                    values.push(data);
                }
                Ok(values)
            }
            async fn find_optional(
                &self,
                query: <#data_path as DataMarker>::Query,
            ) -> Result<Option<Data<#data_path>>, Arc<<#executor_path as DataQueryExecutor<#data_path>>::Error>>
            {
                enum InternalLoadError {
                    Error(<#executor_path as DataQueryExecutor<#data_path>>::Error),
                    NotFound,
                }
                if let Some(id) = self.find_id(&query) {
                    if let Some(data) = self.data.get(&id) {
                        return Ok(Some(data));
                    }
                }
                let fut =  datacache::__internal::FutureExt::map(self.executor.find_optional(query.clone()), |out| {
                    out.map_err(InternalLoadError::Error)
                        .and_then(|opt| match opt {
                            Some(v) => Ok(Data::new(v)),
                            None => Err(InternalLoadError::NotFound),
                        })
                });
                let data = self.query_cache.try_get_with(query, fut).await;
                match data {
                    Ok(data) => {
                        self.insert_data(data.clone()).await;
                        Ok(Some(data))
                    }
                    Err(err) => match &*err {
                        InternalLoadError::NotFound => Ok(None),
                        InternalLoadError::Error(err) => Err(Arc::new(err.clone())),
                    },
                }
            }
            async fn insert(
                &self,
                data: #data_path,
            ) -> Result<(), <#executor_path as DataQueryExecutor<#data_path>>::Error> {
                let data = Data::new(data);
                match self.executor.create(data.clone()).await {
                    Ok(_) => {
                        self.insert_data(data).await;
                        Ok(())
                    }
                    err => err,
                }
            }

            async fn delete(
                &self,
                query: <#data_path as DataMarker>::Query,
            ) -> Result<(), <#executor_path as DataQueryExecutor<#data_path>>::Error> {
                self.query.remove(&query);
                self.query_cache.invalidate(&query).await;
                let ids = self.executor.delete(query).await?;
                for id in ids {
                    self.data.invalidate(&id).await;
                }
                Ok(())
            }
            async fn invalidate(
                &self,
                query: <#data_path as DataMarker>::Query,
            ) -> Result<(), <#executor_path as DataQueryExecutor<#data_path>>::Error> {
                self.query.remove(&query);
                self.query_cache.invalidate(&query).await;
                let ids = self.executor.find_all_ids(query).await?;
                for id in ids {
                    self.data.invalidate(&id).await;
                }
                Ok(())
            }
        }
    };
    Ok(out)
}
