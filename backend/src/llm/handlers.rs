// POST /tasks/generate — сгенерировать задачу; GET /llm/status — подключена ли нейросеть.
use actix_web::{HttpResponse, get, post, web};
use serde::Deserialize;
use serde_json::json;

use super::generator::generate_task;
use crate::auth::AuthUser;
use crate::error::AppError;
use crate::state::AppState;
use crate::tasks::{self, Difficulty, Mode, TaskDetail};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(status).service(generate);
}

#[get("/llm/status")]
async fn status(state: web::Data<AppState>) -> HttpResponse {
    match &state.llm {
        Some(llm) => HttpResponse::Ok().json(json!({ "configured": true, "model": llm.model() })),
        None => HttpResponse::Ok().json(json!({ "configured": false, "model": null })),
    }
}

#[derive(Debug, Deserialize)]
struct GenerateRequest {
    mode: Mode,
    difficulty: Difficulty,
}

// Только для вошедших: у бесплатных API маленькие лимиты, анонимы их быстро съедят
#[post("/tasks/generate")]
async fn generate(
    state: web::Data<AppState>,
    _user: AuthUser,
    body: web::Json<GenerateRequest>,
) -> Result<HttpResponse, AppError> {
    let llm = match &state.llm {
        Some(llm) => llm,
        None => {
            return Err(AppError::Unavailable(
                "Нейросеть не подключена: задай LLM_API_KEY в backend/.env".to_string(),
            ));
        }
    };

    if body.mode == Mode::Css {
        return Err(AppError::BadRequest(
            "Генерация CSS-задач появится вместе с их проверкой".to_string(),
        ));
    }

    // Генерация идёт до минуты и тратит лимит — одновременно только одна
    let _guard = match state.generation_lock.try_lock() {
        Ok(guard) => guard,
        Err(_) => {
            return Err(AppError::Conflict(
                "Нейросеть уже пишет задачу — подожди немного".to_string(),
            ));
        }
    };

    let new_task = generate_task(llm, body.mode, body.difficulty).await?;
    let id = tasks::insert(&state.db, &new_task).await?;
    let task = tasks::find_by_id(&state.db, id).await?;

    log::info!(
        "Нейросеть написала задачу #{id} «{}», бот прошёл {}/{}",
        task.title,
        task.bot_passed,
        task.tests.len()
    );

    Ok(HttpResponse::Ok().json(TaskDetail::from(&task)))
}
