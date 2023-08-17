pub use sea_orm_migration::prelude::*;

mod m20230407_000001_create_test_user_table;
mod m20230817_075937_create_playlist;
mod m20230817_082329_create_artist;
mod m20230817_083218_create_track;
mod m20230817_084239_create_album;
mod m20230817_085519_create_like;
mod m20230817_085837_create_tag;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230407_000001_create_test_user_table::Migration),
            Box::new(m20230817_075937_create_playlist::Migration),
            Box::new(m20230817_082329_create_artist::Migration),
            Box::new(m20230817_083218_create_track::Migration),
            Box::new(m20230817_084239_create_album::Migration),
            Box::new(m20230817_085519_create_like::Migration),
            Box::new(m20230817_085837_create_tag::Migration),
        ]
    }
}
