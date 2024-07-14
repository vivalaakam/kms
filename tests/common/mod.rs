use actix_http::body::to_bytes;
use actix_http::{Request, StatusCode};
use actix_web::dev::{Service, ServiceResponse};
use actix_web::{test, web::Bytes, Error};
use serde::Serialize;

use kms::{AppData, Config};

pub async fn setup() -> AppData {
    let config = Config::default();
    let app_data = AppData::new(&config).await;
    app_data
}

pub async fn post_request_with_data<T, D>(
    app: &T,
    url: &str,
    data: D,
    master_key: Option<&str>,
    secret_key: Option<&str>,
) -> anyhow::Result<(Bytes, StatusCode)>
where
    T: Service<Request, Response = ServiceResponse, Error = Error>,
    D: Serialize,
{
    let mut req = test::TestRequest::post()
        .uri(&format!("/api{url}"))
        .set_json(data);

    if let Some(token) = master_key {
        req = req.insert_header(("x-master-key", token.to_string()))
    }

    if let Some(token) = secret_key {
        req = req.insert_header(("x-secret-key", token.to_string()))
    }

    let resp = app.call(req.to_request()).await.unwrap();

    let status = resp.status();
    let bytes = to_bytes(resp.into_body()).await.unwrap();

    Ok((bytes, status))
}

pub async fn post_request<T>(
    app: &T,
    url: &str,
    master_key: Option<&str>,
    secret_key: Option<&str>,
) -> anyhow::Result<(Bytes, StatusCode)>
where
    T: Service<Request, Response = ServiceResponse, Error = Error>,
{
    let mut req = test::TestRequest::post().uri(&format!("/api{url}"));

    if let Some(token) = master_key {
        req = req.insert_header(("x-master-key", token.to_string()))
    }

    if let Some(token) = secret_key {
        req = req.insert_header(("x-secret-key", token.to_string()))
    }

    let resp = app.call(req.to_request()).await.unwrap();

    let status = resp.status();
    let bytes = to_bytes(resp.into_body()).await.unwrap();

    Ok((bytes, status))
}

pub async fn get_request<T>(
    app: &T,
    url: &str,
    master_key: Option<&str>,
    secret_key: Option<&str>,
) -> anyhow::Result<(Bytes, StatusCode)>
where
    T: Service<Request, Response = ServiceResponse, Error = Error>,
{
    let mut req = test::TestRequest::get().uri(&format!("/api{url}"));

    if let Some(token) = master_key {
        req = req.insert_header(("x-master-key", token.to_string()))
    }

    if let Some(token) = secret_key {
        req = req.insert_header(("x-secret-key", token.to_string()))
    }

    let resp = app.call(req.to_request()).await.unwrap();

    let status = resp.status();
    let bytes = to_bytes(resp.into_body()).await.unwrap();

    Ok((bytes, status))
}
