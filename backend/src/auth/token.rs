// JWT-токен: сервер подписывает id и имя пользователя, клиент присылает токен в заголовке.
// Сессии в базе не храним — подпись сама доказывает, что токен выдали мы.
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

const TOKEN_TTL_SECS: u64 = 7 * 24 * 60 * 60; // неделя

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, // id пользователя
    pub username: String,
    pub exp: u64, // когда токен протухнет, unix-время в секундах
}

pub fn create_token(user_id: i64, username: &str, secret: &str) -> Result<String, AppError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: now + TOKEN_TTL_SECS,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|err| AppError::Internal(format!("jwt: {err}")))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("Сессия истекла — войди заново".to_string()))
}
