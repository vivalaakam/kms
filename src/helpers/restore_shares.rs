use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use num_bigint::BigUint;
use num_traits::Num;
use sea_orm::DbErr;
use thiserror::Error;
use tracing::debug;
use uuid::Uuid;
use vaultrs::error::ClientError;
use vaultrs::kv2;

use crate::models::shares::SharesStatus;
use crate::queries::keys::{get_key_by_id, KeyErrors};
use crate::queries::shares::{get_share_by_secret, ShareErrors};
use crate::services::polynomial::{Share, ShareStore};
use crate::AppData;

#[derive(Debug, Error)]
pub enum RestoreSharesError {
    #[error("Invalid secret key")]
    DecodeError(#[from] base64::DecodeError),
    #[error("Share {0} not found")]
    ShareNotFound(String),
    #[error("Database error: {0}")]
    DbErr(DbErr),
    #[error("Vault error: {0}")]
    Storage(ClientError),
    #[error("Error parsing BigInt: {0}")]
    BigInt(#[from] num_bigint::ParseBigIntError),
    #[error("Key revoked")]
    Revoked,
}

impl From<ShareErrors> for RestoreSharesError {
    fn from(err: ShareErrors) -> Self {
        match err {
            ShareErrors::NotFound(_) => RestoreSharesError::ShareNotFound("Share".to_string()),
            ShareErrors::DbErr(err) => RestoreSharesError::DbErr(err),
        }
    }
}

impl From<KeyErrors> for RestoreSharesError {
    fn from(err: KeyErrors) -> Self {
        match err {
            KeyErrors::NotFound(_) => RestoreSharesError::ShareNotFound("Key".to_string()),
            KeyErrors::DbErr(err) => RestoreSharesError::DbErr(err),
        }
    }
}

pub async fn restore_shares(
    secret_key: &str,
    app_data: &AppData,
) -> Result<(Vec<Share>, Uuid, Uuid), RestoreSharesError> {
    let share = STANDARD
        .decode(secret_key)
        .map_err(RestoreSharesError::DecodeError)?;
    let share_value = hex::encode(share.as_slice());
    debug!("Restoring shares for secret key: {share_value}: {share:?}");
    let share = get_share_by_secret(&share_value, app_data.get_db_connection()).await?;

    if !matches!(share.status, SharesStatus::Granted) {
        return Err(RestoreSharesError::Revoked);
    }

    let key = get_key_by_id(&share.key_id, app_data.get_db_connection()).await?;

    let cloud_secret = kv2::read::<ShareStore>(
        app_data.get_vault_client().as_ref(),
        "secret",
        &key.cloud_key,
    )
    .await
    .map_err(RestoreSharesError::Storage)?;

    let shares = vec![
        Share {
            x: BigUint::from_str_radix(&cloud_secret.x, 16)?,
            y: BigUint::from_str_radix(&cloud_secret.y, 16)?,
        },
        Share {
            x: BigUint::from_str_radix(&key.local_index, 16)?,
            y: BigUint::from_str_radix(&key.local_key, 16)?,
        },
        Share {
            x: BigUint::from_str_radix(&share.user_index, 16)?,
            y: BigUint::from_str_radix(&share_value, 16)?,
        },
    ];

    Ok((shares, share.key_id, share.id))
}
