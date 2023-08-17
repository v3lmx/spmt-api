use sea_orm_migration::prelude::*;

use super::m20230817_082329_create_artist::Artist;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Album::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Album::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Album::SpotifyId).string().not_null())
                    .col(ColumnDef::new(Album::Name).string().not_null())
                    .col(ColumnDef::new(Album::ArtistId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-album-artist_id")
                            .from(Album::Table, Album::ArtistId)
                            .to(Artist::Table, Artist::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Album::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Album {
    Table,
    Id,
    SpotifyId,
    Name,
    ArtistId,
}
