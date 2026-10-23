// Матч против бота:
// POST /matches — начать, GET /matches/{id} — состояние (клиент опрашивает раз в секунду),
// POST /matches/{id}/run | submit | resign, GET /me/matches — история.
use actix_web::{HttpResponse, get, post, web};
use serde_json::json;
use sqlx::SqlitePool;

use super::bot::{bot_rating, build_schedule, progress_at};
use super::model::{CreateMatchRequest, MatchResult, MatchRow, MatchView};
use super::outcome::decide;
use super::rating::league;
use super::repo;
use crate::auth::AuthUser;
use crate::db::now_ms;
use crate::error::AppError;
use crate::runner::{RunRequest, check_solution};
use crate::state::AppState;
use crate::tasks::{self, Difficulty, Mode, TaskDetail};

const COUNTDOWN_MS: i64 = 3_000; // обратный отсчёт перед стартом
const MINUTE_MS: i64 = 60_000;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_match)
        .service(get_match)
        .service(run_code)
        .service(submit_code)
        .service(resign)
        .service(my_matches);
}

fn duration_ms(difficulty: Difficulty) -> i64 {
    match difficulty {
        Difficulty::Easy => 5 * MINUTE_MS,
        Difficulty::Medium => 8 * MINUTE_MS,
        Difficulty::Hard => 12 * MINUTE_MS,
    }
}

#[post("/matches")]
async fn create_match(
    state: web::Data<AppState>,
    user: AuthUser,
    body: web::Json<CreateMatchRequest>,
) -> Result<HttpResponse, AppError> {
    if body.mode == Mode::Css {
        return Err(AppError::BadRequest(
            "Матчи по CSS появятся вместе с проверкой вёрстки".to_string(),
        ));
    }

    let task = match tasks::pick_for_match(&state.db, user.id, body.mode, body.difficulty).await? {
        Some(task) => task,
        None => {
            return Err(AppError::NotFound(
                "Для такого режима и сложности пока нет задач — сгенерируй новую".to_string(),
            ));
        }
    };

    let now = now_ms();
    let starts_at = now + COUNTDOWN_MS;
    let duration = duration_ms(body.difficulty);
    let seed = now as u64 ^ (user.id as u64).rotate_left(32);
    let schedule = build_schedule(
        body.bot_level,
        task.bot_passed,
        task.tests.len() as i64,
        starts_at,
        duration,
        seed,
    );

    let id = repo::insert(
        &state.db,
        user.id,
        &task,
        &body,
        starts_at,
        starts_at + duration,
        &schedule,
    )
    .await?;
    let row = repo::find_for_user(&state.db, id, user.id).await?;

    Ok(HttpResponse::Ok().json(build_view(&state.db, &row, now).await?))
}

#[get("/matches/{id}")]
async fn get_match(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let now = now_ms();
    let row = load_and_settle(&state.db, id.into_inner(), user.id, now).await?;

    Ok(HttpResponse::Ok().json(build_view(&state.db, &row, now).await?))
}

// Только открытые тесты — в зачёт не идёт
#[post("/matches/{id}/run")]
async fn run_code(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<i64>,
    body: web::Json<RunRequest>,
) -> Result<HttpResponse, AppError> {
    let row = load_and_settle(&state.db, id.into_inner(), user.id, now_ms()).await?;
    ensure_playable(&row, now_ms())?;

    let task = tasks::find_by_id(&state.db, row.task_id).await?;
    let result = check_solution(&task, &body.code, false).await?;

    Ok(HttpResponse::Ok().json(result))
}

// Все тесты — это попытка, которая идёт в зачёт
#[post("/matches/{id}/submit")]
async fn submit_code(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<i64>,
    body: web::Json<RunRequest>,
) -> Result<HttpResponse, AppError> {
    let match_id = id.into_inner();
    let row = load_and_settle(&state.db, match_id, user.id, now_ms()).await?;
    ensure_playable(&row, now_ms())?;

    let task = tasks::find_by_id(&state.db, row.task_id).await?;
    let result = check_solution(&task, &body.code, true).await?;

    // Время попытки — момент отправки, а не конец проверки: игрок не должен проигрывать из-за раннера
    let submitted_at = now_ms();
    let passed = result.passed as i64;
    let best = if passed > row.player_passed {
        Some((passed, submitted_at))
    } else {
        None
    };
    repo::record_attempt(&state.db, match_id, best, &body.code).await?;

    let now = now_ms();
    let row = load_and_settle(&state.db, match_id, user.id, now).await?;
    let view = build_view(&state.db, &row, now).await?;

    Ok(HttpResponse::Ok().json(json!({ "run": result, "match": view })))
}

#[post("/matches/{id}/resign")]
async fn resign(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let match_id = id.into_inner();
    let now = now_ms();
    let row = load_and_settle(&state.db, match_id, user.id, now).await?;

    if row.status == "active" {
        repo::finish(&state.db, &row, MatchResult::Loss, now).await?;
    }

    let row = repo::find_for_user(&state.db, match_id, user.id).await?;
    Ok(HttpResponse::Ok().json(build_view(&state.db, &row, now).await?))
}

#[get("/me/matches")]
async fn my_matches(state: web::Data<AppState>, user: AuthUser) -> Result<HttpResponse, AppError> {
    settle_abandoned(&state.db, user.id).await?;
    let items = repo::history(&state.db, user.id).await?;

    Ok(HttpResponse::Ok().json(items))
}

// Загрузить матч и, если по правилам он уже закончился, — завершить и начислить рейтинг
async fn load_and_settle(
    db: &SqlitePool,
    id: i64,
    user_id: i64,
    now: i64,
) -> Result<MatchRow, AppError> {
    let row = repo::find_for_user(db, id, user_id).await?;
    if row.status != "active" {
        return Ok(row);
    }

    match decide(&row, now) {
        Some((result, finished_at)) => {
            repo::finish(db, &row, result, finished_at).await?;
            repo::find_for_user(db, id, user_id).await
        }
        None => Ok(row),
    }
}

// Игрок закрыл вкладку посреди матча — итог всё равно подводится по правилам, когда время вышло.
// Иначе от поражения можно было бы уйти, просто не открывая матч
pub async fn settle_abandoned(db: &SqlitePool, user_id: i64) -> Result<(), AppError> {
    let now = now_ms();

    for row in repo::active_for_user(db, user_id).await? {
        if let Some((result, finished_at)) = decide(&row, now) {
            repo::finish(db, &row, result, finished_at).await?;
        }
    }

    Ok(())
}

fn ensure_playable(row: &MatchRow, now: i64) -> Result<(), AppError> {
    if row.status != "active" {
        return Err(AppError::BadRequest("Матч уже закончился".to_string()));
    }
    if now < row.starts_at {
        return Err(AppError::BadRequest("Матч ещё не начался".to_string()));
    }

    Ok(())
}

async fn build_view(db: &SqlitePool, row: &MatchRow, now: i64) -> Result<MatchView, AppError> {
    let task = tasks::find_by_id(db, row.task_id).await?;

    let status = if row.status != "active" {
        "finished"
    } else if now < row.starts_at {
        "countdown"
    } else {
        "active"
    };

    // Прогресс бота — на текущий момент, а для завершённого матча — на момент конца
    let bot_time = row.finished_at.unwrap_or(now);
    let (bot_passed, _) = progress_at(&row.bot_schedule, bot_time);

    // Рейтинг после матча и эталон — только когда матч закончился
    let (rating_after, solution) = if row.status != "active" {
        let rating = repo::rating(db, row.user_id, row.mode).await?;
        (Some(rating), Some(task.reference_solution()))
    } else {
        (None, None)
    };

    Ok(MatchView {
        id: row.id,
        status,
        mode: row.mode,
        difficulty: row.difficulty,
        bot_level: row.bot_level,
        bot_rating: bot_rating(row.bot_level),
        task: TaskDetail::from(&task),
        server_now: now,
        starts_at: row.starts_at,
        ends_at: row.ends_at,
        total_tests: row.total_tests,
        player_passed: row.player_passed,
        bot_passed,
        result: row.result,
        finished_at: row.finished_at,
        rating_delta: row.rating_delta,
        rating_after,
        league: rating_after.map(league),
        solution,
    })
}
