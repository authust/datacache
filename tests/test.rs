use std::convert::Infallible;
use std::sync::Arc;

use datacache::Data;
use datacache::DataMarker;
use datacache::DataQueryExecutor;
use datacache::DataStorage;

#[test]
fn test_get_storage_by_data() {
    // let mut manager = StorageManager::new();
    // manager.register_storage(TestDataStorage {});
    // assert!(manager.get_storage::<TestDataStorage>().is_some());
    // assert!(manager.get_for_data::<TestData>().is_some());
}

#[derive(DataMarker)]
struct MacroData {
    #[datacache(queryable)]
    id: i32,
    #[datacache(queryable)]
    slug: String,
}

#[derive(DataMarker)]
struct OtherData {
    #[datacache(queryable)]
    id: i32,
    text: String,
}

struct MacroExecutor;
#[datacache::__internal::async_trait]
impl DataQueryExecutor<MacroData> for MacroExecutor {
    type Error = Infallible;
    type Id = i32;
    async fn find_one(&self, _query: MacroDataQuery) -> Result<MacroData, Self::Error> {
        todo!()
    }
    async fn find_all_ids(&self, _query: MacroDataQuery) -> Result<Vec<Self::Id>, Self::Error> {
        todo!()
    }
    async fn find_optional(
        &self,
        _query: MacroDataQuery,
    ) -> Result<Option<MacroData>, Self::Error> {
        todo!()
    }
    async fn create(&self, _data: Data<MacroData>) -> Result<(), Self::Error> {
        todo!()
    }
    async fn update(&self, _data: Data<MacroData>) -> Result<(), Self::Error> {
        todo!()
    }
    async fn delete(&self, _data: MacroDataQuery) -> Result<Vec<Self::Id>, Self::Error> {
        todo!()
    }
}
struct OtherExecutor;
#[datacache::__internal::async_trait]
impl DataQueryExecutor<OtherData> for OtherExecutor {
    type Error = Infallible;
    type Id = i32;
    async fn find_one(&self, _query: OtherDataQuery) -> Result<OtherData, Self::Error> {
        todo!()
    }
    async fn find_all_ids(&self, _query: OtherDataQuery) -> Result<Vec<Self::Id>, Self::Error> {
        todo!()
    }
    async fn find_optional(
        &self,
        _query: OtherDataQuery,
    ) -> Result<Option<OtherData>, Self::Error> {
        todo!()
    }
    async fn create(&self, _data: Data<OtherData>) -> Result<(), Self::Error> {
        todo!()
    }
    async fn update(&self, _data: Data<OtherData>) -> Result<(), Self::Error> {
        todo!()
    }
    async fn delete(&self, _data: OtherDataQuery) -> Result<Vec<Self::Id>, Self::Error> {
        todo!()
    }
}

datacache::storage!(
    MacroDataStorage(MacroExecutor, MacroData),
    id(id: i32),
    unique(),
    fields(d1: String)
);
datacache::storage!(
    OtherDataStorage(OtherExecutor, OtherData),
    id(id: i32),
    unique(),
    fields()
);

datacache::storage_ref!(pub DataRef);
datacache::storage_ref!(MacroData: DataRef where Exc: MacroExecutor, Storage: MacroDataStorage);
datacache::storage_ref!(OtherData: DataRef where Exc: OtherExecutor, Storage: OtherDataStorage);
datacache::storage_manager!(pub DataManager: DataRef);

#[test]
fn test_manager() {
    let mut storage = DataManager::new();
    storage.register_storage(MacroDataStorage::new(MacroExecutor));
    storage.register_storage(OtherDataStorage::new(OtherExecutor));
    assert!(storage.get_for_data::<MacroData>().is_some());
    assert!(storage.get_for_data::<OtherData>().is_some());
}
