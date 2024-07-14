use crate::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("share_owner"))
                    .values([
                        Alias::new("admin"),
                        Alias::new("guest"),
                        Alias::new("unknown"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("share_status"))
                    .values([
                        Alias::new("granted"),
                        Alias::new("revoked"),
                        Alias::new("unknown"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Shares::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Shares::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Shares::KeyId).uuid().not_null())
                    .col(ColumnDef::new(Shares::Secret).string().not_null())
                    .col(ColumnDef::new(Shares::UserIndex).string().not_null())
                    .col(
                        ColumnDef::new(Shares::Owner)
                            .custom(Alias::new("share_owner"))
                            .not_null()
                            .default("unknown"),
                    )
                    .col(
                        ColumnDef::new(Shares::Status)
                            .custom(Alias::new("share_status"))
                            .not_null()
                            .default("unknown"),
                    )
                    .col(
                        ColumnDef::new(Shares::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Shares::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Shares::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Shares {
    Table,
    Id,
    KeyId,
    Secret,
    UserIndex,
    Owner,
    Status,
    CreatedAt,
    UpdatedAt,
}
