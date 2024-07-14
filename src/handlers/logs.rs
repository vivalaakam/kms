use actix_web::{web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::constants::MASTER_KEY;
use crate::queries::keys::{get_key_by_id, KeyErrors};
use crate::queries::logs::get_logs_by_key_id;
use crate::queries::users::{get_user_by_secret, UserErrors};
use crate::AppData;

pub async fn get_logs_handler(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    path: web::Path<Uuid>,
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

    let key = match get_key_by_id(&path.into_inner(), app_data.get_db_connection()).await {
        Ok(key) => key,
        Err(KeyErrors::NotFound(_)) => return HttpResponse::NotFound().finish(),
        Err(_) => {
            return HttpResponse::Unauthorized().finish();
        }
    };

    if key.user_id != user.id {
        return HttpResponse::Unauthorized().finish();
    }

    match get_logs_by_key_id(key.id, app_data.get_db_connection()).await {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
