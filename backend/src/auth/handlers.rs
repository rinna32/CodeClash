// POST /auth/register и /auth/login — оба возвращают токен и пользователя.
use actix_web::{HttpResponse, post, web};
use serde::{Deserialize, Serialize};

use super::password::{hash_password, verify_password};
use super::token::create_token;
use crate::db::now_ms;
use crate::error::AppError;
use crate::state::AppState;

const MIN_USERNAME_LEN: usize = 3;
const MAX_USERNAME_LEN: usize = 20;
const MIN_PASSWORD_LEN: usize = 6;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(register).service(login);
}

#[derive(Debug, Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct UserView {
    pub id: i64,
    pub username: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    token: String,
    user: UserView,
}

#[post("/auth/register")]
async fn register(
    state: web::Data<AppState>,
    body: web::Json<Credentials>,
) -> Result<HttpResponse, AppError> {
    let username = body.username.trim().to_string();
    validate(&username, &body.password)?;

    // argon2 специально медленный (~50 мс) — считаем в отдельном потоке, чтобы не держать сервер
    let password = body.password.clone();
    let password_hash = web::block(move || hash_password(&password))
        .await
        .map_err(|err| AppError::Internal(err.to_string()))??;

    let result =
        sqlx::query("INSERT INTO users (username, password_hash, created_at) VALUES (?, ?, ?)")
            .bind(&username)
            .bind(&password_hash)
            .bind(now_ms())
            .execute(&state.db)
            .await;

    let user_id = match result {
        Ok(result) => result.last_insert_rowid(),
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            return Err(AppError::Conflict("Такое имя уже занято".to_string()));
        }
        Err(err) => return Err(err.into()),
    };

    respond_with_token(&state, user_id, username)
}

#[post("/auth/login")]
async fn login(
    state: web::Data<AppState>,
    body: web::Json<Credentials>,
) -> Result<HttpResponse, AppError> {
    let user: Option<(i64, String, String)> =
        sqlx::query_as("SELECT id, username, password_hash FROM users WHERE username = ?")
            .bind(body.username.trim())
            .fetch_optional(&state.db)
            .await?;

    // Одинаковый ответ для «нет такого» и «не тот пароль» — не подсказываем, какие имена существуют
    let wrong_credentials = || AppError::Unauthorized("Неверное имя или пароль".to_string());

    let (user_id, username, password_hash) = match user {
        Some(user) => user,
        None => return Err(wrong_credentials()),
    };

    let password = body.password.clone();
    let is_valid = web::block(move || verify_password(&password, &password_hash))
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;

    if !is_valid {
        return Err(wrong_credentials());
    }

    respond_with_token(&state, user_id, username)
}

fn validate(username: &str, password: &str) -> Result<(), AppError> {
    let length = username.chars().count();
    if !(MIN_USERNAME_LEN..=MAX_USERNAME_LEN).contains(&length) {
        return Err(AppError::BadRequest(format!(
            "Имя — от {MIN_USERNAME_LEN} до {MAX_USERNAME_LEN} символов"
        )));
    }
    if !username
        .chars()
        .all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(AppError::BadRequest(
            "В имени можно только буквы, цифры, _ и -".to_string(),
        ));
    }
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(AppError::BadRequest(format!(
            "Пароль — минимум {MIN_PASSWORD_LEN} символов"
        )));
    }

    Ok(())
}

fn respond_with_token(
    state: &AppState,
    user_id: i64,
    username: String,
) -> Result<HttpResponse, AppError> {
    let token = create_token(user_id, &username, &state.jwt_secret)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserView {
            id: user_id,
            username,
        },
    }))
}
