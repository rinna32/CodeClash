// HTTP-обработчики задач. Запросы к базе — в repo.rs.
use actix_web::{HttpResponse, get, web};

use super::model::{TaskDetail, TaskFilter};
use super::repo;
use crate::error::AppError;
use crate::state::AppState;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_tasks).service(get_task);
}

#[get("/tasks")]
async fn get_tasks(
    state: web::Data<AppState>,
    filter: web::Query<TaskFilter>,
) -> Result<HttpResponse, AppError> {
    let tasks = repo::find_all(&state.db, &filter).await?;

    Ok(HttpResponse::Ok().json(tasks))
}

#[get("/tasks/{id}")]
async fn get_task(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let task = repo::find_by_id(&state.db, id.into_inner()).await?;

    Ok(HttpResponse::Ok().json(TaskDetail::from(&task)))
}
