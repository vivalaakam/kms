use actix_web::HttpResponse;

pub async fn healthcheck_handler() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}
