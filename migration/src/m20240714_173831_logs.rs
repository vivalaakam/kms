use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Logs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Logs::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Logs::KeyId).uuid().not_null())
                    .col(ColumnDef::new(Logs::Action).string().not_null())
                    .col(ColumnDef::new(Logs::Data).json_binary().not_null())
                    .col(ColumnDef::new(Logs::Message).string())
                    .col(
                        ColumnDef::new(Logs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Logs::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Logs::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Logs {
    Table,
    Id,
    KeyId,
    Action,
    Data,
    Message,
    CreatedAt,
    UpdatedAt,
}
