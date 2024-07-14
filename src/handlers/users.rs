use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::helpers::generate_code::generate_code;
use crate::queries::users::{create_user, CreateOrUpdateUser};
use crate::AppData;

#[derive(Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub secret: String,
}

pub async fn users_create_handler(app_data: web::Data<AppData>) -> HttpResponse {
    let code = generate_code();

    match create_user(
        CreateOrUpdateUser {
            secret: code.clone(),
        },
        app_data.get_db_connection(),
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().json(CreateUserResponse { secret: code }),
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Error creating user: {}", e));
        }
    }
}
