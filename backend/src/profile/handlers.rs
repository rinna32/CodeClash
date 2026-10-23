// GET /me — кто я и мой рейтинг по режимам; GET /leaderboard?mode= — лучшие игроки режима.
use actix_web::{HttpResponse, get, web};
use serde::{Deserialize, Serialize};

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::matches::rating::{START_RATING, league};
use crate::matches::settle_abandoned;
use crate::state::AppState;
use crate::tasks::Mode;

const LEADERBOARD_SIZE: i64 = 20;
const RATED_MODES: [Mode; 2] = [Mode::Javascript, Mode::Python]; // CSS-матчей пока нет

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(me).service(leaderboard);
}

#[derive(Debug, Serialize)]
struct ModeRating {
    mode: Mode,
    rating: i64,
    games: i64,
    wins: i64,
    league: &'static str,
}

#[derive(Debug, Serialize)]
struct MeResponse {
    id: i64,
    username: String,
    ratings: Vec<ModeRating>,
}

#[get("/me")]
async fn me(state: web::Data<AppState>, user: AuthUser) -> Result<HttpResponse, AppError> {
    settle_abandoned(&state.db, user.id).await?;

    let rows: Vec<(Mode, i64, i64, i64)> =
        sqlx::query_as("SELECT mode, rating, games, wins FROM ratings WHERE user_id = ?")
            .bind(user.id)
            .fetch_all(&state.db)
            .await?;

    // Режимы, в которых ещё не играл, показываем со стартовым рейтингом
    let ratings = RATED_MODES
        .iter()
        .map(|mode| {
            let (rating, games, wins) = rows
                .iter()
                .find(|row| row.0 == *mode)
                .map(|row| (row.1, row.2, row.3))
                .unwrap_or((START_RATING, 0, 0));

            ModeRating {
                mode: *mode,
                rating,
                games,
                wins,
                league: league(rating),
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(MeResponse {
        id: user.id,
        username: user.username,
        ratings,
    }))
}

#[derive(Debug, Deserialize)]
struct LeaderboardQuery {
    mode: Mode,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct LeaderboardRow {
    username: String,
    rating: i64,
    games: i64,
    wins: i64,
}

#[derive(Debug, Serialize)]
struct LeaderboardItem {
    place: usize,
    username: String,
    rating: i64,
    games: i64,
    wins: i64,
    league: &'static str,
}

#[get("/leaderboard")]
async fn leaderboard(
    state: web::Data<AppState>,
    query: web::Query<LeaderboardQuery>,
) -> Result<HttpResponse, AppError> {
    let rows = sqlx::query_as::<_, LeaderboardRow>(
        "SELECT u.username, r.rating, r.games, r.wins
         FROM ratings r JOIN users u ON u.id = r.user_id
         WHERE r.mode = ? AND r.games > 0
         ORDER BY r.rating DESC, r.wins DESC LIMIT ?",
    )
    .bind(query.mode)
    .bind(LEADERBOARD_SIZE)
    .fetch_all(&state.db)
    .await?;

    let items: Vec<LeaderboardItem> = rows
        .into_iter()
        .enumerate()
        .map(|(index, row)| LeaderboardItem {
            place: index + 1,
            league: league(row.rating),
            username: row.username,
            rating: row.rating,
            games: row.games,
            wins: row.wins,
        })
        .collect();

    Ok(HttpResponse::Ok().json(items))
}
