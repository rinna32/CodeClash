// AuthUser — текущий пользователь. Достаточно добавить его аргументом в обработчик:
// actix сам проверит заголовок Authorization и вернёт 401, если токена нет, он плохой
// или такого пользователя уже нет в базе.
use std::future::Future;
use std::pin::Pin;

use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest, web};

use super::token::verify_token;
use crate::error::AppError;
use crate::state::AppState;

pub struct AuthUser {
    pub id: i64,
    pub username: String,
}

impl FromRequest for AuthUser {
    type Error = AppError;
    type Future = Pin<Box<dyn Future<Output = Result<AuthUser, AppError>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move { authenticate(&req).await })
    }
}

async fn authenticate(req: &HttpRequest) -> Result<AuthUser, AppError> {
    let state = match req.app_data::<web::Data<AppState>>() {
        Some(state) => state.clone(),
        None => return Err(AppError::Internal("AppState не подключён".to_string())),
    };

    // Заголовок вида: Authorization: Bearer <токен>
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    let token = match token {
        Some(token) => token,
        None => return Err(AppError::Unauthorized("Нужно войти в аккаунт".to_string())),
    };

    let claims = verify_token(token, &state.jwt_secret)?;

    // Токен может быть честно подписан, но пользователя уже нет (например, базу удалили).
    // Без этой проверки запросы падали бы с ошибкой базы вместо понятного «войди заново»
    let username: Option<String> = sqlx::query_scalar("SELECT username FROM users WHERE id = ?")
        .bind(claims.sub)
        .fetch_optional(&state.db)
        .await?;

    match username {
        Some(username) => Ok(AuthUser {
            id: claims.sub,
            username,
        }),
        None => Err(AppError::Unauthorized(
            "Сессия истекла — войди заново".to_string(),
        )),
    }
}
