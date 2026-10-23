// GET /tasks/{id}/solution — сдаться на тренировке и посмотреть эталон,
// POST /tasks/{id}/review — разбор кода на тренировке,
// POST /matches/{id}/review — разбор после матча; сохраняется, чтобы не тратить лимит повторно.
use actix_web::{HttpResponse, get, post, web};
use serde::Deserialize;
use serde_json::json;

use super::repo;
use crate::auth::AuthUser;
use crate::db::now_ms;
use crate::error::AppError;
use crate::llm::{LlmClient, review_solution};
use crate::state::AppState;
use crate::tasks;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(task_solution)
        .service(task_review)
        .service(match_review);
}

#[derive(Debug, Deserialize)]
struct ReviewRequest {
    code: String,
}

// Вход нужен, чтобы закрыть ответ на время матча: иначе его можно подсмотреть в соседней вкладке
#[get("/tasks/{id}/solution")]
async fn task_solution(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let task = tasks::find_by_id(&state.db, id.into_inner()).await?;
    ensure_not_in_match(&state, user.id, task.id).await?;

    Ok(HttpResponse::Ok().json(json!({ "solution": task.reference_solution() })))
}

#[post("/tasks/{id}/review")]
async fn task_review(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<i64>,
    body: web::Json<ReviewRequest>,
) -> Result<HttpResponse, AppError> {
    let llm = require_llm(&state)?;
    let task = tasks::find_by_id(&state.db, id.into_inner()).await?;
    ensure_not_in_match(&state, user.id, task.id).await?;

    let review = review_solution(llm, &task, &body.code).await?;

    Ok(HttpResponse::Ok().json(review))
}

#[post("/matches/{id}/review")]
async fn match_review(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<i64>,
    body: web::Json<ReviewRequest>,
) -> Result<HttpResponse, AppError> {
    let match_id = id.into_inner();
    let row = repo::find_match(&state.db, match_id, user.id).await?;

    if !row.is_finished {
        return Err(AppError::BadRequest(
            "Разбор появится, когда матч закончится".to_string(),
        ));
    }
    if let Some(review) = row.review {
        return Ok(HttpResponse::Ok().json(review));
    }

    let llm = require_llm(&state)?;
    let task = tasks::find_by_id(&state.db, row.task_id).await?;
    let review = review_solution(llm, &task, &body.code).await?;
    repo::save_match_review(&state.db, match_id, &review).await?;

    Ok(HttpResponse::Ok().json(review))
}

fn require_llm(state: &AppState) -> Result<&LlmClient, AppError> {
    match &state.llm {
        Some(llm) => Ok(llm),
        None => Err(AppError::Unavailable(
            "Нейросеть не подключена — разбор недоступен".to_string(),
        )),
    }
}

async fn ensure_not_in_match(state: &AppState, user_id: i64, task_id: i64) -> Result<(), AppError> {
    if repo::is_in_match(&state.db, user_id, task_id, now_ms()).await? {
        return Err(AppError::BadRequest(
            "Эта задача сейчас у тебя в матче — ответ будет после его конца".to_string(),
        ));
    }

    Ok(())
}
