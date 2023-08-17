use sea_orm_migration::prelude::*;

use super::m20230407_000001_create_test_user_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Playlist::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Playlist::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Playlist::SpotifyId).string().not_null())
                    .col(ColumnDef::new(Playlist::Name).string().not_null())
                    .col(ColumnDef::new(Playlist::OwnerId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-playlist-user_id")
                            .from(Playlist::Table, Playlist::OwnerId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Playlist::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Playlist {
    Table,
    Id,
    SpotifyId,
    Name,
    OwnerId,
}
