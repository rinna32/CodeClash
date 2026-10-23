// Запросы к таблице tasks.
use sqlx::SqlitePool;
use sqlx::types::Json;

use super::model::{Difficulty, Mode, NewTask, Task, TaskFilter, TaskListItem};
use crate::db::now_ms;
use crate::error::AppError;

const TASK_COLUMNS: &str = "id, mode, difficulty, title, statement_md, function_name, params, tests, \
                            solution, bot_solution, bot_passed, target_html, source";

pub async fn find_all(db: &SqlitePool, filter: &TaskFilter) -> Result<Vec<TaskListItem>, AppError> {
    // Один запрос на все комбинации фильтра: NULL в параметре означает «не фильтровать»
    let tasks = sqlx::query_as::<_, TaskListItem>(
        "SELECT id, mode, difficulty, title, source FROM tasks
         WHERE (?1 IS NULL OR mode = ?1) AND (?2 IS NULL OR difficulty = ?2)
         ORDER BY id DESC",
    )
    .bind(filter.mode)
    .bind(filter.difficulty)
    .fetch_all(db)
    .await?;

    Ok(tasks)
}

pub async fn find_by_id(db: &SqlitePool, id: i64) -> Result<Task, AppError> {
    let task = sqlx::query_as::<_, Task>(&format!("SELECT {TASK_COLUMNS} FROM tasks WHERE id = ?"))
        .bind(id)
        .fetch_optional(db)
        .await?;

    match task {
        Some(task) => Ok(task),
        None => Err(AppError::NotFound("Задача не найдена".to_string())),
    }
}

// Задача для матча: сначала те, что игрок ещё не встречал, среди них — случайная.
// Нужна задача, которую бот уже решил: без его решения играть не с кем.
pub async fn pick_for_match(
    db: &SqlitePool,
    user_id: i64,
    mode: Mode,
    difficulty: Difficulty,
) -> Result<Option<Task>, AppError> {
    let task = sqlx::query_as::<_, Task>(&format!(
        "SELECT {TASK_COLUMNS} FROM tasks
         WHERE mode = ?1 AND difficulty = ?2 AND bot_solution IS NOT NULL
         ORDER BY id IN (SELECT task_id FROM matches WHERE user_id = ?3), RANDOM()
         LIMIT 1"
    ))
    .bind(mode)
    .bind(difficulty)
    .bind(user_id)
    .fetch_optional(db)
    .await?;

    Ok(task)
}

pub async fn insert(db: &SqlitePool, task: &NewTask) -> Result<i64, AppError> {
    let result = sqlx::query(
        "INSERT INTO tasks (mode, difficulty, title, statement_md, function_name, params, tests,
                            solution, bot_solution, bot_passed, target_html, source, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(task.mode)
    .bind(task.difficulty)
    .bind(&task.title)
    .bind(&task.statement_md)
    .bind(&task.function_name)
    .bind(Json(&task.params))
    .bind(Json(&task.tests))
    .bind(&task.solution)
    .bind(&task.bot_solution)
    .bind(task.bot_passed)
    .bind(&task.target_html)
    .bind(task.source)
    .bind(now_ms())
    .execute(db)
    .await?;

    Ok(result.last_insert_rowid())
}
