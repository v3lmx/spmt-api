use sea_orm_migration::prelude::*;

use super::m20230407_000001_create_test_user_table::User;
use super::m20230817_083218_create_track::Track;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Like::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Like::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Like::UserId).uuid().not_null())
                    .col(ColumnDef::new(Like::TrackId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-like-user_id")
                            .from(Like::Table, Like::UserId)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-like-track_id")
                            .from(Like::Table, Like::TrackId)
                            .to(Track::Table, Track::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Like::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Like {
    Table,
    Id,
    UserId,
    TrackId,
}
