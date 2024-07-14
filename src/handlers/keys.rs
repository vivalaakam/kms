use actix_web::{web, HttpRequest, HttpResponse};
use alloy::signers::local::PrivateKeySigner;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use num_bigint::BigUint;
use num_traits::Num;
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;
use vaultrs::kv2;

use crate::constants::{MASTER_KEY, SECRET_KEY};
use crate::helpers::generate_code::{generate_code, generate_random};
use crate::models::shares::SharesOwner;
use crate::queries::keys::{create_key, get_key_by_id, CreateOrUpdateKey, KeyErrors};
use crate::queries::logs::{create_log, CreateLog};
use crate::queries::shares::{
    create_share, get_share_by_id, get_share_by_secret, revoke_share_by_id, CreateOrUpdateShare,
    ShareErrors,
};
use crate::queries::users::{get_user_by_secret, UserErrors};
use crate::services::polynomial::{Polynomial, Share, ShareStore};
use crate::AppData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeysGenerateResponse {
    pub key: String,
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeysRevokeRequest {
    pub id: Uuid,
}

pub async fn keys_generate_handler(req: HttpRequest, app_data: web::Data<AppData>) -> HttpResponse {
    let Some(Ok(master_key)) = req.headers().get(MASTER_KEY).map(|header| header.to_str()) else {
        return HttpResponse::Unauthorized().finish();
    };

    let user = match get_user_by_secret(&master_key, app_data.get_db_connection()).await {
        Ok(user) => user,
        Err(UserErrors::NotFound(_)) => {
            return HttpResponse::Unauthorized().finish();
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Error getting user: {}", e));
        }
    };

    let private_key = generate_random();
    let Ok(signer) = PrivateKeySigner::from_slice(private_key.as_slice()) else {
        return HttpResponse::InternalServerError().finish();
    };

    let secret = BigUint::from_bytes_be(private_key.as_slice());

    let poly = Polynomial::new();

    let shares = poly
        .generate_shares(&secret, 5, 3)
        .iter()
        .map(Into::into)
        .collect::<Vec<ShareStore>>();

    debug!("Shares: {:?}", shares);

    let path = generate_code();

    if let Err(err) = kv2::set(
        app_data.get_vault_client().as_ref(),
        "secret",
        &path,
        &shares[0],
    )
    .await
    {
        return HttpResponse::InternalServerError().body(format!("Error setting secret: {}", err));
    }

    let key = CreateOrUpdateKey {
        user_id: user.id,
        local_key: shares[1].y.clone(),
        local_index: shares[1].y.clone(),
        cloud_key: path,
        address: signer.address(),
    };

    let key = match create_key(key, app_data.get_db_connection()).await {
        Ok(key) => key,
        Err(err) => {
            return HttpResponse::InternalServerError()
                .body(format!("Error creating key: {}", err));
        }
    };

    debug!("Share key: {}", shares[2].y.clone());

    let share = match create_share(
        CreateOrUpdateShare {
            secret: shares[2].y.clone(),
            key_id: key.id,
            user_index: shares[2].x.clone(),
            owner: SharesOwner::Admin,
        },
        app_data.get_db_connection(),
    )
    .await
    {
        Ok(share) => share,
        Err(err) => {
            return HttpResponse::InternalServerError()
                .body(format!("Error creating share: {}", err));
        }
    };

    let Ok(user_key) = hex::decode(&shares[2].y) else {
        return HttpResponse::InternalServerError().finish();
    };

    debug!("Generate key: {:?}", key);

    let _ = create_log(
        CreateLog {
            key_id: key.id,
            action: "generate_key".to_string(),
            data: serde_json::json!({
                "user_id": user.id
            }),
            message: None,
        },
        app_data.get_db_connection(),
    )
    .await;

    HttpResponse::Ok().json(KeysGenerateResponse {
        key: STANDARD.encode(user_key),
        id: share.id,
    })
}

pub async fn keys_grant_handler(req: HttpRequest, app_data: web::Data<AppData>) -> HttpResponse {
    let Some(Ok(master_key)) = req.headers().get(MASTER_KEY).map(|header| header.to_str()) else {
        return HttpResponse::Unauthorized().finish();
    };

    let Some(Ok(secret_key)) = req.headers().get(SECRET_KEY).map(|header| header.to_str()) else {
        return HttpResponse::Unauthorized().finish();
    };

    let user = match get_user_by_secret(&master_key, app_data.get_db_connection()).await {
        Ok(user) => user,
        Err(UserErrors::NotFound(_)) => {
            return HttpResponse::Unauthorized().finish();
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Error getting user: {}", e));
        }
    };

    let Ok(share) = STANDARD.decode(secret_key) else {
        return HttpResponse::Unauthorized().finish();
    };

    let share_value = hex::encode(share);
    let share = match get_share_by_secret(&share_value, app_data.get_db_connection()).await {
        Ok(share) => share,
        Err(ShareErrors::NotFound(_)) => return HttpResponse::NotFound().finish(),
        Err(_) => {
            return HttpResponse::Unauthorized().finish();
        }
    };

    let key = match get_key_by_id(&share.key_id, app_data.get_db_connection()).await {
        Ok(key) => key,
        Err(KeyErrors::NotFound(_)) => return HttpResponse::NotFound().finish(),
        Err(_) => {
            return HttpResponse::Unauthorized().finish();
        }
    };

    if key.user_id != user.id {
        return HttpResponse::Unauthorized().finish();
    }

    let Ok(cloud_secret) = kv2::read::<ShareStore>(
        app_data.get_vault_client().as_ref(),
        "secret",
        &key.cloud_key,
    )
    .await
    else {
        return HttpResponse::InternalServerError().finish();
    };

    let shares = vec![
        Share {
            x: BigUint::from_str_radix(&cloud_secret.x, 16).expect("Error parsing local index"),
            y: BigUint::from_str_radix(&cloud_secret.y, 16).expect("Error parsing local key"),
        },
        Share {
            x: BigUint::from_str_radix(&key.local_index, 16).expect("Error parsing local index"),
            y: BigUint::from_str_radix(&key.local_key, 16).expect("Error parsing local key"),
        },
        Share {
            x: BigUint::from_str_radix(&share.user_index, 16).expect("Error parsing user index"),
            y: BigUint::from_str_radix(&share_value, 16).expect("Error parsing user key"),
        },
    ];

    let sss = Polynomial::new();

    let new_share = ShareStore::from(sss.add_share(&shares));

    let share = match create_share(
        CreateOrUpdateShare {
            secret: new_share.y.to_string(),
            key_id: key.id,
            user_index: new_share.x.to_string(),
            owner: SharesOwner::Guest,
        },
        app_data.get_db_connection(),
    )
    .await
    {
        Ok(share) => share,
        Err(err) => {
            return HttpResponse::InternalServerError()
                .body(format!("Error creating share: {}", err));
        }
    };

    let Ok(user_key) = hex::decode(&new_share.y).map(|k| STANDARD.encode(k)) else {
        return HttpResponse::InternalServerError().finish();
    };

    let _ = create_log(
        CreateLog {
            key_id: key.id,
            action: "grant".to_string(),
            data: serde_json::json!({
                "user_id": user.id,
                "share_id": share.id,
            }),
            message: None,
        },
        app_data.get_db_connection(),
    )
    .await;

    HttpResponse::Ok().json(KeysGenerateResponse {
        key: user_key,
        id: share.id,
    })
}

pub async fn keys_revoke_handler(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    body: web::Json<KeysRevokeRequest>,
) -> HttpResponse {
    let Some(Ok(master_key)) = req.headers().get(MASTER_KEY).map(|header| header.to_str()) else {
        return HttpResponse::Unauthorized().finish();
    };

    let user = match get_user_by_secret(&master_key, app_data.get_db_connection()).await {
        Ok(user) => user,
        Err(UserErrors::NotFound(_)) => {
            return HttpResponse::Unauthorized().finish();
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Error getting user: {}", e));
        }
    };

    let share = match get_share_by_id(&body.id, app_data.get_db_connection()).await {
        Ok(share) => share,
        Err(ShareErrors::NotFound(_)) => return HttpResponse::NotFound().finish(),
        Err(_) => {
            return HttpResponse::Unauthorized().finish();
        }
    };

    let key = match get_key_by_id(&share.key_id, app_data.get_db_connection()).await {
        Ok(key) => key,
        Err(KeyErrors::NotFound(_)) => return HttpResponse::NotFound().finish(),
        Err(_) => {
            return HttpResponse::Unauthorized().finish();
        }
    };

    if key.user_id != user.id {
        return HttpResponse::Unauthorized().finish();
    }

    if revoke_share_by_id(&share.id, app_data.get_db_connection())
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    let _ = create_log(
        CreateLog {
            key_id: key.id,
            action: "revoke".to_string(),
            data: serde_json::json!({
                "user_id": user.id,
                "share_id": share.id,
            }),
            message: None,
        },
        app_data.get_db_connection(),
    )
    .await;

    HttpResponse::Ok().finish()
}
