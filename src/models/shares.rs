use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "shares")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub key_id: Uuid,
    pub secret: String,
    pub owner: SharesOwner,
    pub status: SharesStatus,
    pub user_index: String,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, Default, Serialize, Deserialize, EnumIter, DeriveActiveEnum, PartialEq)]
#[serde(rename_all = "camelCase")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "share_owner")]
pub enum SharesOwner {
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "guest")]
    Guest,
    #[default]
    #[sea_orm(string_value = "unknown")]
    Unknown,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, EnumIter, DeriveActiveEnum, PartialEq)]
#[serde(rename_all = "camelCase")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "share_status")]
pub enum SharesStatus {
    #[sea_orm(string_value = "granted")]
    Granted,
    #[sea_orm(string_value = "revoked")]
    Revoked,
    #[default]
    #[sea_orm(string_value = "unknown")]
    Unknown,
}
