// Запросы для разбора: сохранённый разбор матча и проверка «не идёт ли матч на этой задаче».
use sqlx::SqlitePool;
use sqlx::types::Json;

use crate::error::AppError;
use crate::llm::Review;

pub struct MatchReviewRow {
    pub task_id: i64,
    pub is_finished: bool,
    pub review: Option<Review>,
}

// Матч только своего игрока: чужой выглядит как несуществующий
pub async fn find_match(
    db: &SqlitePool,
    id: i64,
    user_id: i64,
) -> Result<MatchReviewRow, AppError> {
    let row: Option<(i64, String, Option<Json<Review>>)> =
        sqlx::query_as("SELECT task_id, status, review FROM matches WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .fetch_optional(db)
            .await?;

    match row {
        Some((task_id, status, review)) => Ok(MatchReviewRow {
            task_id,
            is_finished: status == "finished",
            review: review.map(|review| review.0),
        }),
        None => Err(AppError::NotFound("Матч не найден".to_string())),
    }
}

pub async fn save_match_review(db: &SqlitePool, id: i64, review: &Review) -> Result<(), AppError> {
    sqlx::query("UPDATE matches SET review = ? WHERE id = ?")
        .bind(Json(review))
        .bind(id)
        .execute(db)
        .await?;

    Ok(())
}

// Идёт ли у игрока матч на этой задаче прямо сейчас
pub async fn is_in_match(
    db: &SqlitePool,
    user_id: i64,
    task_id: i64,
    now: i64,
) -> Result<bool, AppError> {
    let in_match: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM matches
                        WHERE user_id = ? AND task_id = ? AND status = 'active' AND ends_at > ?)",
    )
    .bind(user_id)
    .bind(task_id)
    .bind(now)
    .fetch_one(db)
    .await?;

    Ok(in_match)
}
