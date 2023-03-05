use std::convert::Infallible;
use std::fmt::Debug;
use std::fmt::Display;

use datacache::Data;
use datacache::DataMarker;
use datacache::DataQueryExecutor;
use datacache::DataRef;
use datacache::LookupRef;

#[test]
fn test_get_storage_by_data() {
    // let mut manager = StorageManager::new();
    // manager.register_storage(TestDataStorage {});
    // assert!(manager.get_storage::<TestDataStorage>().is_some());
    // assert!(manager.get_for_data::<TestData>().is_some());
}

#[derive(DataMarker, Debug, PartialEq, Eq)]
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
}

struct MacroExecutor;
#[datacache::__internal::async_trait]
impl DataQueryExecutor<MacroData> for MacroExecutor {
    type Error = Infallible;
    type Id = i32;
    fn get_id(&self, data: &MacroData) -> Self::Id {
        data.id
    }
    async fn find_one(&self, query: &MacroDataQuery) -> Result<MacroData, Self::Error> {
        self.find_optional(query)
            .await
            .map(|opt| opt.expect("Not found"))
    }
    async fn find_all_ids(
        &self,
        _query: Option<&MacroDataQuery>,
    ) -> Result<Vec<Self::Id>, Self::Error> {
        todo!()
    }
    async fn find_optional(
        &self,
        query: &MacroDataQuery,
    ) -> Result<Option<MacroData>, Self::Error> {
        if let MacroDataQuery::id(id) = query {
            if id == &7 {
                Ok(Some(MacroData {
                    id: 7,
                    slug: "Test Data".into(),
                }))
            } else {
                panic!("Only id 7 lookup");
            }
        } else {
            panic!("Slug lookup not tested")
        }
    }
    async fn create(&self, _data: Data<MacroData>) -> Result<(), Self::Error> {
        todo!()
    }
    async fn update(&self, _data: Data<MacroData>) -> Result<(), Self::Error> {
        todo!()
    }
    async fn delete(&self, _data: &MacroDataQuery) -> Result<Vec<Self::Id>, Self::Error> {
        todo!()
    }
}
struct OtherExecutor;
#[datacache::__internal::async_trait]
impl DataQueryExecutor<OtherData> for OtherExecutor {
    type Error = Infallible;
    type Id = i32;
    fn get_id(&self, data: &OtherData) -> Self::Id {
        data.id
    }
    async fn find_one(&self, _query: &OtherDataQuery) -> Result<OtherData, Self::Error> {
        todo!()
    }
    async fn find_all_ids(
        &self,
        _query: Option<&OtherDataQuery>,
    ) -> Result<Vec<Self::Id>, Self::Error> {
        todo!()
    }
    async fn find_optional(
        &self,
        _query: &OtherDataQuery,
    ) -> Result<Option<OtherData>, Self::Error> {
        todo!()
    }
    async fn create(&self, _data: Data<OtherData>) -> Result<(), Self::Error> {
        todo!()
    }
    async fn update(&self, _data: Data<OtherData>) -> Result<(), Self::Error> {
        todo!()
    }
    async fn delete(&self, _data: &OtherDataQuery) -> Result<Vec<Self::Id>, Self::Error> {
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

datacache::storage_ref!(pub StorageRef);
datacache::storage_ref!(MacroData: StorageRef where Exc: MacroExecutor, Storage: MacroDataStorage);
datacache::storage_ref!(OtherData: StorageRef where Exc: OtherExecutor, Storage: OtherDataStorage);
datacache::storage_manager!(pub DataManager: StorageRef, handle_error);

fn handle_error(err: impl Display) {
    println!("An error occurred {err}")
}

fn manager() -> DataManager {
    let mut storage = DataManager::new();
    storage.register_storage(MacroDataStorage::new(MacroExecutor));
    storage.register_storage(OtherDataStorage::new(OtherExecutor));
    storage
}

#[test]
fn test_manager() {
    let storage = manager();
    assert!(storage.get_for_data::<MacroData>().is_some());
    assert!(storage.get_for_data::<OtherData>().is_some());
}

#[tokio::test]
async fn test_lookup() {
    let storage = manager();
    let d_ref: DataRef<MacroData> = DataRef::new(MacroDataQuery::id(7));
    let data = storage.lookup(&d_ref).await;
    assert_eq!(
        Some(Data::new(MacroData {
            id: 7,
            slug: "Test Data".into(),
        })),
        data,
    );
}
