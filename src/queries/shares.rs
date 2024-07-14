use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::instrument;
use uuid::Uuid;

use crate::helpers::keccak256::keccak256;
use crate::models::shares::{ActiveModel, Column, Entity, Model, SharesOwner, SharesStatus};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateOrUpdateShare {
    pub secret: String,
    pub key_id: Uuid,
    pub user_index: String,
    pub owner: SharesOwner,
}

#[derive(Debug, Error)]
pub enum ShareErrors {
    #[error("Share not found: {0}")]
    NotFound(String),
    #[error("DbErr: {0}")]
    DbErr(DbErr),
}

#[instrument(level = "debug", name = "create_share", skip(connection))]
pub async fn create_share<D>(
    data: CreateOrUpdateShare,
    connection: &D,
) -> Result<Model, ShareErrors>
where
    D: ConnectionTrait,
{
    let model = ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        key_id: ActiveValue::Set(data.key_id),
        secret: ActiveValue::Set(keccak256(data.secret.clone())),
        user_index: ActiveValue::Set(data.user_index),
        owner: ActiveValue::Set(data.owner),
        status: ActiveValue::Set(SharesStatus::Granted),
        created_at: ActiveValue::Set(Utc::now().into()),
        updated_at: ActiveValue::Set(Utc::now().into()),
    };

    model.insert(connection).await.map_err(ShareErrors::DbErr)
}

#[instrument(level = "debug", name = "get_share_by_secret", skip(connection))]
pub async fn get_share_by_secret<D>(secret: &str, connection: &D) -> Result<Model, ShareErrors>
where
    D: ConnectionTrait,
{
    match Entity::find()
        .filter(Column::Secret.eq(keccak256(secret.to_string())))
        .one(connection)
        .await
    {
        Ok(Some(client)) => Ok(client),
        Ok(None) => Err(ShareErrors::NotFound("hidden secret".to_string())),
        Err(err) => Err(ShareErrors::DbErr(err)),
    }
}

#[instrument(level = "debug", name = "get_share_by_id", skip(connection))]
pub async fn get_share_by_id<D>(id: &Uuid, connection: &D) -> Result<Model, ShareErrors>
where
    D: ConnectionTrait,
{
    match Entity::find_by_id(*id).one(connection).await {
        Ok(Some(client)) => Ok(client),
        Ok(None) => Err(ShareErrors::NotFound(id.to_string())),
        Err(err) => Err(ShareErrors::DbErr(err)),
    }
}

#[instrument(level = "debug", name = "revoke_share_by_id", skip(connection))]
pub async fn revoke_share_by_id<D>(id: &Uuid, connection: &D) -> Result<Model, ShareErrors>
where
    D: ConnectionTrait,
{
    let model = get_share_by_id(id, connection).await?;

    let mut row: ActiveModel = model.into();

    row.status = ActiveValue::Set(SharesStatus::Revoked);
    row.updated_at = ActiveValue::Set(Utc::now().into());

    row.update(connection).await.map_err(ShareErrors::DbErr)
}
