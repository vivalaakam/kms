use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use dotenv::dotenv;
use tracing::info;
use tracing::level_filters::LevelFilter;

use kms::{handlers, AppData, Config};
use migration::{Migrator, MigratorTrait};

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .with_test_writer()
        .init();

    let config = Config::default();

    let app_data = AppData::new(&config).await;

    Migrator::up(app_data.get_db_connection(), None)
        .await
        .expect("migration error");

    let port = config.clone().port.unwrap_or(String::from("8080"));

    info!("Starting web_app on port: {port}");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().starts_with(b"http://localhost")
            })
            .allowed_origin(
                config
                    .cors_origin_url
                    .as_ref()
                    .unwrap_or(&"http://localhost".to_string()),
            )
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(app_data.clone()))
            .configure(handlers)
    })
    .bind(format!("0.0.0.0:{port}"))
    .expect("panic")
    .run()
    .await
    .expect("http_server error");
}
