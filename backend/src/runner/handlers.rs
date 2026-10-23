// POST /tasks/{id}/run и /tasks/{id}/submit — тренировка без соперника, вход не нужен.
use actix_web::{HttpResponse, post, web};

use super::check::check_solution;
use super::model::RunRequest;
use crate::error::AppError;
use crate::state::AppState;
use crate::tasks;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(run_task).service(submit_task);
}

// Только открытые тесты: игрок проверяет себя
#[post("/tasks/{id}/run")]
async fn run_task(
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<RunRequest>,
) -> Result<HttpResponse, AppError> {
    let task = tasks::find_by_id(&state.db, id.into_inner()).await?;
    let result = check_solution(&task, &body.code, false).await?;

    Ok(HttpResponse::Ok().json(result))
}

// Все тесты, включая скрытые
#[post("/tasks/{id}/submit")]
async fn submit_task(
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<RunRequest>,
) -> Result<HttpResponse, AppError> {
    let task = tasks::find_by_id(&state.db, id.into_inner()).await?;
    let result = check_solution(&task, &body.code, true).await?;

    Ok(HttpResponse::Ok().json(result))
}
