use serde::{Deserialize, Serialize};

use crate::tasks::{Difficulty, Mode, TaskDetail};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum BotLevel {
    Novice,
    Middle,
    Senior,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum MatchResult {
    Win,
    Loss,
    Draw,
}

// Один «Submit» бота: в момент at он прошёл passed тестов
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BotStep {
    pub at: i64,
    pub passed: i64,
}

// Строка таблицы matches. bot_schedule клиенту не отдаём — иначе было бы видно, когда бот решит
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MatchRow {
    pub id: i64,
    pub user_id: i64,
    pub task_id: i64,
    pub mode: Mode,
    pub difficulty: Difficulty,
    pub bot_level: BotLevel,
    pub total_tests: i64,
    pub starts_at: i64,
    pub ends_at: i64,
    #[sqlx(json)]
    pub bot_schedule: Vec<BotStep>,
    pub player_passed: i64,
    pub player_best_at: Option<i64>,
    pub status: String, // active | finished
    pub result: Option<MatchResult>,
    pub finished_at: Option<i64>,
    pub rating_delta: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMatchRequest {
    pub mode: Mode,
    pub difficulty: Difficulty,
    pub bot_level: BotLevel,
}

// ── Что видит клиент ──

#[derive(Debug, Serialize)]
pub struct MatchView {
    pub id: i64,
    pub status: &'static str, // countdown | active | finished
    pub mode: Mode,
    pub difficulty: Difficulty,
    pub bot_level: BotLevel,
    pub bot_rating: i64,
    pub task: TaskDetail,
    pub server_now: i64, // по нему клиент выравнивает свои часы
    pub starts_at: i64,
    pub ends_at: i64,
    pub total_tests: i64,
    pub player_passed: i64,
    pub bot_passed: i64, // сколько бот прошёл к текущему моменту
    pub result: Option<MatchResult>,
    pub finished_at: Option<i64>, // таймер на клиенте замирает на этом моменте
    pub rating_delta: Option<i64>,
    pub rating_after: Option<i64>,
    pub league: Option<&'static str>,
    pub solution: Option<String>, // эталон — только после конца матча, иначе его можно было бы списать
}

// Элемент истории матчей
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MatchHistoryItem {
    pub id: i64,
    pub task_title: String,
    pub mode: Mode,
    pub difficulty: Difficulty,
    pub bot_level: BotLevel,
    pub player_passed: i64,
    pub total_tests: i64,
    pub result: Option<MatchResult>,
    pub rating_delta: Option<i64>,
    pub finished_at: Option<i64>,
}
