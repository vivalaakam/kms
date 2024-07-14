use actix_web::web;

pub use keys::{KeysGenerateResponse, KeysRevokeRequest};
pub use sign::{SignMessageRequest, SignMessageResponse};
pub use users::CreateUserResponse;

mod healthcheck;
mod keys;
mod logs;
mod sign;
mod users;

pub fn handlers(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(web::resource("/users").route(web::post().to(users::users_create_handler)))
        .service(web::resource("/logs/{id}").route(web::get().to(logs::get_logs_handler)))
        .service(web::resource("/keys/generate").route(web::post().to(keys::keys_generate_handler)))
        .service(web::resource("/keys/grant").route(web::post().to(keys::keys_grant_handler)))
        .service(web::resource("/keys/revoke").route(web::post().to(keys::keys_revoke_handler)))
        .service(web::resource("/sign_message").route(web::post().to(sign::sign_message_handler)));

    conf.service(scope);
    conf.service(web::resource("/").route(web::get().to(healthcheck::healthcheck_handler)));
}
