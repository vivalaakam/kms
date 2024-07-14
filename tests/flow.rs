use actix_http::StatusCode;
use actix_web::{test, web, App};
use tracing::level_filters::LevelFilter;
use tracing::warn;

use kms::{
    handlers, CreateUserResponse, KeysGenerateResponse, KeysRevokeRequest, SignMessageRequest,
    SignMessageResponse,
};
use migration::{Migrator, MigratorTrait};

use crate::common::{get_request, post_request, post_request_with_data};

mod common;

#[tokio::test]
async fn test_flow() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .with_test_writer()
        .init();

    let app_data = common::setup().await;

    Migrator::up(app_data.get_db_connection(), None)
        .await
        .expect("migration error");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .configure(handlers),
    )
    .await;

    let (resp, status) = post_request(&app, "/users", None, None).await.unwrap();

    assert_eq!(status, StatusCode::OK);

    let CreateUserResponse { secret } =
        serde_json::from_slice(&resp).expect("Failed to parse response");
    assert!(secret.len() > 0);

    let (resp, status) = post_request(&app, "/keys/generate", Some(&secret), None)
        .await
        .unwrap();

    assert_eq!(status, StatusCode::OK);

    let KeysGenerateResponse {
        key: key_share_a,
        id: _,
    } = serde_json::from_slice(&resp).expect("Failed to parse response");

    warn!("Key #a: {key_share_a}");

    let message = "Hello, world!";

    let (resp, status) = post_request_with_data(
        &app,
        "/sign_message",
        Some(SignMessageRequest {
            message: message.to_string(),
        }),
        None,
        Some(&key_share_a),
    )
    .await
    .unwrap();

    assert_eq!(status, StatusCode::OK);

    let SignMessageResponse {
        signature: signature_1,
    } = serde_json::from_slice(&resp).expect("Failed to parse response");

    println!("Signature 1: {signature_1}");

    let (resp, status) = post_request(&app, "/keys/grant", Some(&secret), Some(&key_share_a))
        .await
        .unwrap();

    assert_eq!(status, StatusCode::OK);

    let KeysGenerateResponse {
        key: key_share_b,
        id: id_share_b,
    } = serde_json::from_slice(&resp).expect("Failed to parse response");

    warn!("Key #b: {key_share_b}");

    assert_ne!(key_share_a, key_share_b);

    let (resp, status) = post_request_with_data(
        &app,
        "/sign_message",
        Some(SignMessageRequest {
            message: message.to_string(),
        }),
        None,
        Some(&key_share_b),
    )
    .await
    .unwrap();

    assert_eq!(status, StatusCode::OK);

    let SignMessageResponse {
        signature: signature_2,
    } = serde_json::from_slice(&resp).expect("Failed to parse response");

    println!("Signature 1: {signature_2}");

    assert_eq!(signature_1, signature_2);

    let (_resp, status) = post_request_with_data(
        &app,
        "/keys/revoke",
        Some(KeysRevokeRequest { id: id_share_b }),
        Some(&secret),
        Some(&key_share_a),
    )
    .await
    .unwrap();

    assert_eq!(status, StatusCode::OK);

    let (_resp, status) = post_request_with_data(
        &app,
        "/sign_message",
        Some(SignMessageRequest {
            message: message.to_string(),
        }),
        None,
        Some(&key_share_b),
    )
    .await
    .unwrap();

    assert_eq!(status, StatusCode::BAD_REQUEST);
}
