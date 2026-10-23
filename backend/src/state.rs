// Общее состояние, которое actix раздаёт всем обработчикам.
use sqlx::SqlitePool;

use crate::llm::LlmClient;

pub struct AppState {
    pub db: SqlitePool,
    pub jwt_secret: String,
    pub llm: Option<LlmClient>, // None — ключ нейросети не задан, генерация выключена
    pub generation_lock: tokio::sync::Mutex<()>, // одна генерация за раз
}
