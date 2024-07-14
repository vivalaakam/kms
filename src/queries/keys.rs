use alloy::primitives::Address;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, DbErr, EntityTrait};
use thiserror::Error;
use tracing::instrument;
use uuid::Uuid;

use crate::models::keys::{ActiveModel, Entity, Model};

#[derive(Debug, Error)]
pub enum KeyErrors {
    #[error("Key not found: {0}")]
    NotFound(String),
    #[error("DbErr: {0}")]
    DbErr(DbErr),
}

#[derive(Debug)]
pub struct CreateOrUpdateKey {
    pub user_id: Uuid,
    pub local_key: String,
    pub local_index: String,
    pub cloud_key: String,
    pub address: Address,
}

#[instrument(level = "debug", name = "create_key", skip(connection))]
pub async fn create_key<D>(data: CreateOrUpdateKey, connection: &D) -> Result<Model, KeyErrors>
where
    D: ConnectionTrait,
{
    let model = ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        user_id: ActiveValue::Set(data.user_id),
        local_key: ActiveValue::Set(data.local_key),
        local_index: ActiveValue::Set(data.local_index),
        cloud_key: ActiveValue::Set(data.cloud_key),
        address: ActiveValue::Set(data.address.to_string()),
        created_at: ActiveValue::Set(Utc::now().into()),
        updated_at: ActiveValue::Set(Utc::now().into()),
    };

    model.insert(connection).await.map_err(KeyErrors::DbErr)
}

#[instrument(level = "debug", name = "get_key_by_id", skip(connection))]
pub async fn get_key_by_id<D>(id: &Uuid, connection: &D) -> Result<Model, KeyErrors>
where
    D: ConnectionTrait,
{
    match Entity::find_by_id(*id).one(connection).await {
        Ok(Some(client)) => Ok(client),
        Ok(None) => Err(KeyErrors::NotFound(id.to_string())),
        Err(err) => Err(KeyErrors::DbErr(err)),
    }
}
