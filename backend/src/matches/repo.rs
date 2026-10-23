// Запросы к таблицам matches и ratings.
use sqlx::SqlitePool;
use sqlx::types::Json;

use super::bot::bot_rating;
use super::model::{BotStep, CreateMatchRequest, MatchHistoryItem, MatchResult, MatchRow};
use super::rating::{START_RATING, rating_delta};
use crate::error::AppError;
use crate::tasks::{Mode, Task};

const MATCH_COLUMNS: &str = "id, user_id, task_id, mode, difficulty, bot_level, total_tests, starts_at, ends_at, \
                             bot_schedule, player_passed, player_best_at, status, result, finished_at, rating_delta";
const HISTORY_LIMIT: i64 = 50;

pub async fn insert(
    db: &SqlitePool,
    user_id: i64,
    task: &Task,
    request: &CreateMatchRequest,
    starts_at: i64,
    ends_at: i64,
    schedule: &[BotStep],
) -> Result<i64, AppError> {
    let result = sqlx::query(
        "INSERT INTO matches (user_id, task_id, mode, difficulty, bot_level, total_tests, starts_at, ends_at, bot_schedule)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(task.id)
    .bind(request.mode)
    .bind(request.difficulty)
    .bind(request.bot_level)
    .bind(task.tests.len() as i64)
    .bind(starts_at)
    .bind(ends_at)
    .bind(Json(schedule))
    .execute(db)
    .await?;

    Ok(result.last_insert_rowid())
}

// Матч только своего игрока: чужой выглядит как несуществующий
pub async fn find_for_user(db: &SqlitePool, id: i64, user_id: i64) -> Result<MatchRow, AppError> {
    let row = sqlx::query_as::<_, MatchRow>(&format!(
        "SELECT {MATCH_COLUMNS} FROM matches WHERE id = ? AND user_id = ?"
    ))
    .bind(id)
    .bind(user_id)
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(row),
        None => Err(AppError::NotFound("Матч не найден".to_string())),
    }
}

// Новый лучший результат игрока; код сохраняем всегда — для истории
pub async fn record_attempt(
    db: &SqlitePool,
    id: i64,
    best: Option<(i64, i64)>, // (passed, at), если результат улучшился
    code: &str,
) -> Result<(), AppError> {
    match best {
        Some((passed, at)) => {
            sqlx::query("UPDATE matches SET player_passed = ?, player_best_at = ?, player_code = ? WHERE id = ?")
                .bind(passed)
                .bind(at)
                .bind(code)
                .bind(id)
                .execute(db)
                .await?;
        }
        None => {
            sqlx::query("UPDATE matches SET player_code = ? WHERE id = ?")
                .bind(code)
                .bind(id)
                .execute(db)
                .await?;
        }
    }

    Ok(())
}

// Завершение матча и пересчёт рейтинга — в одной транзакции.
// WHERE status = 'active' защищает от двойного начисления, если два запроса придут одновременно.
pub async fn finish(
    db: &SqlitePool,
    row: &MatchRow,
    result: MatchResult,
    finished_at: i64,
) -> Result<(), AppError> {
    let mut tx = db.begin().await?;

    let player_rating = rating_in(&mut tx, row.user_id, row.mode).await?;
    let score = match result {
        MatchResult::Win => 1.0,
        MatchResult::Draw => 0.5,
        MatchResult::Loss => 0.0,
    };
    let delta = rating_delta(player_rating, bot_rating(row.bot_level), score);

    let updated = sqlx::query(
        "UPDATE matches SET status = 'finished', result = ?, finished_at = ?, rating_delta = ?
         WHERE id = ? AND status = 'active'",
    )
    .bind(result)
    .bind(finished_at)
    .bind(delta)
    .bind(row.id)
    .execute(&mut *tx)
    .await?;

    if updated.rows_affected() == 0 {
        return Ok(()); // матч уже завершил параллельный запрос
    }

    sqlx::query(
        "INSERT INTO ratings (user_id, mode, rating, games, wins) VALUES (?1, ?2, ?3, 1, ?4)
         ON CONFLICT (user_id, mode) DO UPDATE SET rating = ?3, games = games + 1, wins = wins + ?4",
    )
    .bind(row.user_id)
    .bind(row.mode)
    .bind(player_rating + delta)
    .bind(if result == MatchResult::Win { 1 } else { 0 })
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn rating(db: &SqlitePool, user_id: i64, mode: Mode) -> Result<i64, AppError> {
    let mut conn = db.acquire().await?;
    rating_in(&mut conn, user_id, mode).await
}

async fn rating_in(
    conn: &mut sqlx::SqliteConnection,
    user_id: i64,
    mode: Mode,
) -> Result<i64, AppError> {
    let rating: Option<(i64,)> =
        sqlx::query_as("SELECT rating FROM ratings WHERE user_id = ? AND mode = ?")
            .bind(user_id)
            .bind(mode)
            .fetch_optional(conn)
            .await?;

    Ok(rating.map(|(rating,)| rating).unwrap_or(START_RATING))
}

// Незавершённые матчи игрока — чтобы подвести итог тем, из которых он просто ушёл
pub async fn active_for_user(db: &SqlitePool, user_id: i64) -> Result<Vec<MatchRow>, AppError> {
    let rows = sqlx::query_as::<_, MatchRow>(&format!(
        "SELECT {MATCH_COLUMNS} FROM matches WHERE user_id = ? AND status = 'active'"
    ))
    .bind(user_id)
    .fetch_all(db)
    .await?;

    Ok(rows)
}

pub async fn history(db: &SqlitePool, user_id: i64) -> Result<Vec<MatchHistoryItem>, AppError> {
    let items = sqlx::query_as::<_, MatchHistoryItem>(
        "SELECT m.id, t.title AS task_title, m.mode, m.difficulty, m.bot_level, m.player_passed,
                m.total_tests, m.result, m.rating_delta, m.finished_at
         FROM matches m JOIN tasks t ON t.id = m.task_id
         WHERE m.user_id = ? AND m.status = 'finished'
         ORDER BY m.id DESC LIMIT ?",
    )
    .bind(user_id)
    .bind(HISTORY_LIMIT)
    .fetch_all(db)
    .await?;

    Ok(items)
}
