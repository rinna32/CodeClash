// Единый тип ошибки API: обработчик возвращает Result<HttpResponse, AppError>,
// а клиент всегда получает JSON вида { "error": "..." }.
use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    Unavailable(String), // сервис выключен, например не задан ключ нейросети

    #[error("Нейросеть ответила с ошибкой: {0}")]
    Llm(String),

    #[error("Ошибка базы данных")]
    Database(#[from] sqlx::Error), // #[from] даёт писать `?` после запросов sqlx

    #[error("Внутренняя ошибка сервера")]
    Internal(String), // текст — только для лога
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Llm(_) => StatusCode::BAD_GATEWAY,
            AppError::Database(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        // Детали внутренних ошибок пишем только в лог: клиенту незачем видеть SQL и системные сообщения
        match self {
            AppError::Internal(details) => log::error!("{details}"),
            AppError::Database(err) => log::error!("Ошибка базы: {err}"),
            AppError::Llm(details) => log::warn!("Нейросеть: {details}"),
            _ => {}
        }

        HttpResponse::build(self.status_code()).json(json!({ "error": self.to_string() }))
    }
}
