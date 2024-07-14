use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Keys::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Keys::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Keys::UserId).uuid().not_null())
                    .col(ColumnDef::new(Keys::Address).string().not_null())
                    .col(ColumnDef::new(Keys::LocalKey).string().not_null())
                    .col(ColumnDef::new(Keys::LocalIndex).string().not_null())
                    .col(ColumnDef::new(Keys::CloudKey).string().not_null())
                    .col(
                        ColumnDef::new(Keys::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Keys::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Keys::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Keys {
    Table,
    Id,
    UserId,
    LocalKey,
    LocalIndex,
    CloudKey,
    Address,
    CreatedAt,
    UpdatedAt,
}
