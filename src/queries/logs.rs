use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter,
};
use serde_json::Value;
use thiserror::Error;
use tracing::instrument;
use uuid::Uuid;

use crate::models::logs::{ActiveModel, Column, Entity, Model};

#[derive(Debug)]
pub struct CreateLog {
    pub key_id: Uuid,
    pub action: String,
    pub data: Value,
    pub message: Option<String>,
}

#[derive(Debug, Error)]
pub enum LogErrors {
    #[error("Log not found: {0}")]
    NotFound(String),
    #[error("DbErr: {0}")]
    DbErr(DbErr),
}

#[instrument(level = "debug", name = "create_log", skip(connection))]
pub async fn create_log<D>(data: CreateLog, connection: &D) -> Result<Model, LogErrors>
where
    D: ConnectionTrait,
{
    let model = ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        key_id: ActiveValue::Set(data.key_id),
        action: ActiveValue::Set(data.action),
        data: ActiveValue::Set(data.data),
        message: ActiveValue::Set(data.message),
        created_at: ActiveValue::Set(Utc::now().into()),
        updated_at: ActiveValue::Set(Utc::now().into()),
    };

    model.insert(connection).await.map_err(LogErrors::DbErr)
}

pub async fn get_logs_by_key_id<D>(key_id: Uuid, connection: &D) -> Result<Vec<Model>, LogErrors>
where
    D: ConnectionTrait,
{
    Entity::find()
        .filter(Column::KeyId.eq(key_id))
        .all(connection)
        .await
        .map_err(LogErrors::DbErr)
}
