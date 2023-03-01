use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Pointer},
    hash::Hash,
    ops::Deref,
    sync::Arc,
};

pub use derive::DataMarker;

#[doc(hidden)]
pub use async_trait::async_trait as __internal_async_trait;

#[doc(hidden)]
pub use dashmap as __internal_dashmap;
#[doc(hidden)]
pub use futures_util as __internal_futures_util;
#[doc(hidden)]
pub use moka as __internal_moka;

#[doc(hidden)]
pub use derive::storage as __internal_storage_macro;

#[macro_export]
macro_rules! storage {
    ($vis:vis $ident:ident($exc:ty, $data:ty), id($id_field:ident: $id_ty:ty), unique($($unique:ident: $unique_ty:ty),* ), fields($($field:ident: $field_ty:ty),* )) => {
        $crate::__internal_storage_macro!($vis $ident($exc, $data), id($id_field: $id_ty), unique($($unique: $unique_ty),*), fields($($field: $field_ty),*));
    };
}

pub trait DataMarker {
    type Query;

    fn create_queries(&self) -> Vec<Self::Query>;
}

pub trait DataStorageRef {
    type Exc: DataQueryExecutor<Self::Data>;
    type Data: DataMarker;
    type Storage: DataStorage<Self::Exc, Self::Data>;
}

#[repr(transparent)]
pub struct Data<T>(Arc<T>);

impl<T: DataMarker> Data<T> {
    pub fn new(data: T) -> Self {
        Self(Arc::new(data))
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
            type Storage: datacache::DataStorage<Self::Exc, Self>;
        }
    };
    ($data:path: $ref:ident where Exc: $exc:path, Storage: $storage:path) => {
        impl $ref for $data {
            type Exc = $exc;
            type Storage = $storage;
        }
    }
}

pub struct DataRef<D: DataMarker>(D::Query);

impl<D: DataMarker> DataRef<D> {
    pub fn new(query: D::Query) -> Self {
        Self(query)
    }
}

#[async_trait::async_trait]
pub trait DataQueryExecutor<D: DataMarker>: Sized {
    type Error;
    type Id;
    async fn find_one(&self, query: D::Query) -> Result<D, Self::Error>;
    // async fn find_all(&self, query: D::Query) -> Result<Vec<D>, Self::Error>;
    async fn find_all_ids(&self, query: D::Query) -> Result<Vec<Self::Id>, Self::Error>;
    async fn find_optional(&self, query: D::Query) -> Result<Option<D>, Self::Error>;
    async fn create(&self, data: Data<D>) -> Result<(), Self::Error>;
    async fn update(&self, data: Data<D>) -> Result<(), Self::Error>;
    async fn delete(&self, data: D::Query) -> Result<Vec<Self::Id>, Self::Error>;
}

#[async_trait::async_trait]
pub trait DataStorage<Exc: DataQueryExecutor<D>, D: DataMarker> {
    async fn find_one(&self, query: D::Query) -> Result<Data<D>, Arc<Exc::Error>>;
    async fn find_all(&self, query: D::Query) -> Result<Vec<Data<D>>, Arc<Exc::Error>>;
    async fn find_optional(&self, query: D::Query) -> Result<Option<Data<D>>, Arc<Exc::Error>>;

    async fn insert(&self, data: D) -> Result<(), Exc::Error>;

    async fn delete(&self, query: D::Query) -> Result<(), Exc::Error>;
    async fn invalidate(&self, query: D::Query) -> Result<(), Exc::Error>;
}

#[macro_export]
macro_rules! storage_manager {
    ($vis:vis $ident:ident: $ref:path) => {
        $vis struct $ident {
            storage: std::collections::HashMap<std::any::TypeId, std::sync::Arc<dyn std::any::Any>>,
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
                T: DataStorage<Exc, D> + 'static,
                Exc: DataQueryExecutor<D> + 'static,
                D: datacache::DataMarker + 'static,
            >(
                &mut self,
                storage: T,
            ) {
                let id = std::any::TypeId::of::<T>();
                self.data.insert(std::any::TypeId::of::<D>(), id.clone());
                self.storage.insert(id, Arc::new(storage));
            }
        }
    };
}
