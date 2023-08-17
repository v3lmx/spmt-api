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
                    .table(Track::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Track::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Track::SpotifyId).string().not_null())
                    .col(ColumnDef::new(Track::Name).string().not_null())
                    .col(ColumnDef::new(Track::ArtistId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-track-artist_id")
                            .from(Track::Table, Track::ArtistId)
                            .to(Artist::Table, Artist::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Track::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub(crate) enum Track {
    Table,
    Id,
    SpotifyId,
    Name,
    ArtistId,
}
