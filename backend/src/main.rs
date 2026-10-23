// Точка входа API CodeClash: настройки, база, нейросеть и HTTP-сервер.
mod auth;
mod config;
mod db;
mod error;
mod health;
mod llm;
mod matches;
mod profile;
mod review;
mod runner;
mod state;
mod tasks;

use actix_cors::Cors;
use actix_web::{App, HttpServer, http::header, middleware::Logger, web};

use crate::config::Config;
use crate::error::AppError;
use crate::llm::LlmClient;
use crate::state::AppState;

const CORS_MAX_AGE: usize = 3600; // с, браузер кеширует preflight на час

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // .env может не быть (например, переменные заданы снаружи) — это не ошибка
    dotenvy::dotenv().ok();
    env_logger::init();

    let config = Config::from_env();
    let frontend_origin = config.frontend_origin.clone();
    let db = db::connect(&config.database_url).await;

    match &config.llm {
        Some(llm) => log::info!("Нейросеть подключена: {} ({})", llm.model, llm.base_url),
        None => log::warn!(
            "Нейросеть не подключена — задай LLM_API_KEY в .env, чтобы генерировать задачи"
        ),
    }

    // web::Data — общий для всех потоков сервера указатель на состояние
    let state = web::Data::new(AppState {
        db,
        jwt_secret: config.jwt_secret.clone(),
        llm: config.llm.clone().map(LlmClient::new),
        generation_lock: tokio::sync::Mutex::new(()),
    });

    log::info!("API слушает http://{}:{}", config.host, config.port);

    HttpServer::new(move || {
        // Фронт живёт на другом порту, без CORS браузер не даст ему обращаться к API
        let cors = Cors::default()
            .allowed_origin(&frontend_origin)
            .allowed_methods(["GET", "POST"])
            .allowed_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
            .max_age(CORS_MAX_AGE);

        // Без этого actix отвечает на кривой ?mode=..., id или тело простым текстом, а фронту нужен JSON
        let query_config = web::QueryConfig::default()
            .error_handler(|err, _| AppError::BadRequest(err.to_string()).into());
        let path_config = web::PathConfig::default()
            .error_handler(|err, _| AppError::BadRequest(err.to_string()).into());
        let json_config = web::JsonConfig::default()
            .error_handler(|err, _| AppError::BadRequest(err.to_string()).into());

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(state.clone())
            .app_data(query_config)
            .app_data(path_config)
            .app_data(json_config)
            .service(
                web::scope("/api")
                    .configure(health::routes)
                    .configure(auth::routes)
                    .configure(profile::routes)
                    .configure(llm::routes) // до tasks: иначе /tasks/generate спутается с /tasks/{id}
                    .configure(tasks::routes)
                    .configure(runner::routes)
                    .configure(matches::routes)
                    .configure(review::routes),
            )
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}
