use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::instrument;
use uuid::Uuid;

use crate::helpers::keccak256::keccak256;
use crate::models::users::{ActiveModel, Column, Entity, Model};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateOrUpdateUser {
    pub secret: String,
}

#[derive(Debug, Error)]
pub enum UserErrors {
    #[error("User not found: {0}")]
    NotFound(String),
    #[error("DbErr: {0}")]
    DbErr(DbErr),
}

#[instrument(level = "debug", name = "create_user", skip(connection))]
pub async fn create_user<D>(data: CreateOrUpdateUser, connection: &D) -> Result<Model, UserErrors>
where
    D: ConnectionTrait,
{
    let secret = keccak256(data.secret.clone());

    let model = ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        secret: ActiveValue::Set(secret),
        created_at: ActiveValue::Set(Utc::now().into()),
        updated_at: ActiveValue::Set(Utc::now().into()),
    };

    model.insert(connection).await.map_err(UserErrors::DbErr)
}

#[instrument(level = "debug", name = "get_user_by_secret", skip(connection))]
pub async fn get_user_by_secret<D>(secret: &str, connection: &D) -> Result<Model, UserErrors>
where
    D: ConnectionTrait,
{
    match Entity::find()
        .filter(Column::Secret.eq(keccak256(secret.to_string())))
        .one(connection)
        .await
    {
        Ok(Some(client)) => Ok(client),
        Ok(None) => Err(UserErrors::NotFound("hidden secret".to_string())),
        Err(err) => Err(UserErrors::DbErr(err)),
    }
}
