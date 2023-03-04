use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Pointer},
    hash::Hash,
    ops::Deref,
    sync::Arc,
};

pub use derive::DataMarker;

#[doc(hidden)]
pub mod __internal {
    pub use async_trait::async_trait;
    pub use dashmap;
    pub use derive::storage;
    pub use futures_util::FutureExt;
    pub use moka;
    #[cfg(feature = "serde")]
    pub use serde::Deserialize;
    #[cfg(feature = "serde")]
    pub use serde::Serialize;
}

#[macro_export]
macro_rules! storage {
    ($vis:vis $ident:ident($exc:ty, $data:ty), id($id_field:ident: $id_ty:ty), unique($($unique:ident: $unique_ty:ty),* ), fields($($field:ident: $field_ty:ty),* )) => {
        $crate::__internal::storage!($vis $ident($exc, $data), id($id_field: $id_ty), unique($($unique: $unique_ty),*), fields($($field: $field_ty),*));
    };
}

pub trait DataMarker {
    type Query: Send + Sync + Hash + Eq;

    fn create_queries(&self) -> Vec<Self::Query>;
}

#[repr(transparent)]
pub struct Data<T>(Arc<T>);

impl<T: DataMarker> Data<T> {
    pub fn new(data: T) -> Self {
        Self(Arc::new(data))
    }
}

impl<T: DataMarker> DataMarker for Data<T> {
    type Query = T::Query;

    fn create_queries(&self) -> Vec<Self::Query> {
        T::create_queries(&self.0)
    }
}

impl<T> Deref for Data<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> AsRef<T> for Data<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T> Borrow<T> for Data<T> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}

impl<T> Clone for Data<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T: Debug> Debug for Data<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}
impl<T: Display> Display for Data<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
impl<T: Hash> Hash for Data<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}
impl<T: PartialEq> PartialEq<Data<T>> for Data<T> {
    fn eq(&self, other: &Data<T>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T: Eq> Eq for Data<T> {}

impl<T: PartialOrd> PartialOrd<Data<T>> for Data<T> {
    fn partial_cmp(&self, other: &Data<T>) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: Ord> Ord for Data<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl<T: Pointer> Pointer for Data<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[macro_export]
macro_rules! storage_ref {
    ($vis:vis $ident:ident) => {
        $vis trait $ident: datacache::DataMarker + Sized {
            type Exc: datacache::DataQueryExecutor<Self>;
            type Storage: datacache::DataStorage<Self::Exc, Self> + Clone;
        }
    };
    ($data:path: $ref:ident where Exc: $exc:path, Storage: $storage:path) => {
        impl $ref for $data {
            type Exc = $exc;
            type Storage = $storage;
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DataRef<D: DataMarker>(pub D::Query);

impl<D> Debug for DataRef<D>
where
    D: DataMarker,
    D::Query: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DataRef").field(&self.0).finish()
    }
}
impl<D> Display for DataRef<D>
where
    D: DataMarker,
    D::Query: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<D> Clone for DataRef<D>
where
    D: DataMarker,
    D::Query: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<D: DataMarker> DataRef<D> {
    pub fn new(query: D::Query) -> Self {
        Self(query)
    }
}

#[async_trait::async_trait]
pub trait DataQueryExecutor<D: DataMarker>: Sized + Send + Sync {
    type Error: Display;
    type Id: Send + Sync + Hash + Eq + Clone;

    fn get_id(&self, data: &D) -> Self::Id;
    async fn find_one(&self, query: D::Query) -> Result<D, Self::Error>;
    // async fn find_all(&self, query: D::Query) -> Result<Vec<D>, Self::Error>;
    async fn find_all_ids(&self, query: D::Query) -> Result<Vec<Self::Id>, Self::Error>;
    async fn find_optional(&self, query: D::Query) -> Result<Option<D>, Self::Error>;
    async fn create(&self, data: Data<D>) -> Result<(), Self::Error>;
    async fn update(&self, data: Data<D>) -> Result<(), Self::Error>;
    async fn delete(&self, data: D::Query) -> Result<Vec<Self::Id>, Self::Error>;
}

#[async_trait::async_trait]
pub trait DataStorage<Exc: DataQueryExecutor<D>, D: DataMarker>: Send + Sync {
    async fn find_one(&self, query: D::Query) -> Result<Data<D>, Arc<Exc::Error>>;
    async fn find_all(&self, query: D::Query) -> Result<Vec<Data<D>>, Arc<Exc::Error>>;
    async fn find_optional(&self, query: D::Query) -> Result<Option<Data<D>>, Arc<Exc::Error>>;

    async fn insert(&self, data: D) -> Result<(), Exc::Error>;

    async fn delete(&self, query: D::Query) -> Result<(), Exc::Error>;
    async fn invalidate(&self, query: D::Query) -> Result<(), Exc::Error>;

    fn get_executor(&self) -> &Exc;
}

#[async_trait::async_trait]
pub trait LookupRef<D: DataMarker> {
    async fn lookup(&self, reference: DataRef<D>) -> Option<Data<D>>;
}

#[macro_export]
macro_rules! storage_manager {
    ($vis:vis $ident:ident: $ref:path, $lookup_ref_handle_error:ident) => {
        $vis struct $ident {
            storage: std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any + Send + Sync>>,
            data: std::collections::HashMap<std::any::TypeId, std::any::TypeId>,
        }

        impl $ident {
            pub fn new() -> Self {
                Self {
                    storage: std::collections::HashMap::new(),
                    data: std::collections::HashMap::new(),
                }
            }

            pub fn get_for_data<D: $ref + 'static>(&self) -> Option<&D::Storage> {
                let Some(id) = self.data.get(&std::any::TypeId::of::<D>()) else { return None };
                self.storage
                    .get(id)
                    .map(|v| v.downcast_ref::<D::Storage>())
                    .flatten()
            }

            pub fn get_storage<T: 'static>(&self) -> Option<&T> {
                let id = std::any::TypeId::of::<T>();
                self.storage
                    .get(&id)
                    .map(|v| v.downcast_ref::<T>())
                    .flatten()
            }
            pub fn register_storage<
                T: datacache::DataStorage<Exc, D> + 'static,
                Exc: datacache::DataQueryExecutor<D> + 'static,
                D: datacache::DataMarker + 'static,
            >(
                &mut self,
                storage: T,
            ) {
                let id = std::any::TypeId::of::<T>();
                self.data.insert(std::any::TypeId::of::<D>(), id.clone());
                self.storage.insert(id, Box::new(storage));
            }

            pub fn get_and_remove<S: 'static>(&mut self) -> Option<Box<S>> {
                self.storage.remove(&std::any::TypeId::of::<S>()).map(|v| v.downcast::<S>().expect("Downcast failed"))
            }
        }
        #[datacache::__internal::async_trait]
        impl<D: $ref + 'static> datacache::LookupRef<D> for $ident {
            async fn lookup(&self, reference: datacache::DataRef<D>) -> Option<Data<D>>{
                let storage = self.get_for_data::<D>();
                match storage {
                    Some(storage) => {
                        let res = datacache::DataStorage::find_optional(storage, reference.0).await;
                        match res {
                            Ok(value) => value,
                            Err(err) => {
                                $lookup_ref_handle_error(err);
                                None
                            }
                        }
                    },
                    None => None,
                }
            }
        }
    };
}
