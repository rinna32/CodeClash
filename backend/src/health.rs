// Проверка, что сервер жив. Нужна фронту и для деплоя.
use actix_web::{HttpResponse, Responder, get, web};
use serde_json::json;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(health);
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(json!({ "status": "ok" }))
}
