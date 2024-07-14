use actix_web::{web, HttpRequest, HttpResponse};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::constants::SECRET_KEY;
use crate::helpers::restore_shares::restore_shares;
use crate::queries::logs::{create_log, CreateLog};
use crate::services::polynomial::Polynomial;
use crate::AppData;

#[derive(Deserialize, Serialize, Debug)]
pub struct SignMessageRequest {
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SignMessageResponse {
    pub signature: String,
}

pub async fn sign_message_handler(
    app_data: web::Data<AppData>,
    req: HttpRequest,
    body: web::Json<SignMessageRequest>,
) -> HttpResponse {
    let Some(Ok(secret_key)) = req.headers().get(SECRET_KEY).map(|header| header.to_str()) else {
        return HttpResponse::Unauthorized().finish();
    };

    let (shares, key_id, share_id) = match restore_shares(secret_key, &app_data).await {
        Ok(shares) => shares,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({"error": e.to_string()}));
        }
    };

    let sss = Polynomial::new();

    let private_key = sss.reconstruct_secret(&shares);

    let Ok(signer) = PrivateKeySigner::from_slice(private_key.to_bytes_be().as_slice()) else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok(signature) = signer.sign_message(body.message.as_bytes()).await else {
        return HttpResponse::InternalServerError().finish();
    };

    let _ = create_log(
        CreateLog {
            key_id,
            action: "sign_message".to_string(),
            data: json!({
                "share_id": share_id,
            }),
            message: Some(body.message.clone()),
        },
        app_data.get_db_connection(),
    )
    .await;

    HttpResponse::Ok().json(SignMessageResponse {
        signature: hex::encode(signature.as_bytes()),
    })
}
