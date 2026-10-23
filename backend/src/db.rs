// Подключение к SQLite, миграции и стартовые задачи.
use std::str::FromStr;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use crate::tasks;

const DB_MAX_CONNECTIONS: u32 = 5; // SQLite пишет в один поток, больше соединений не ускорят

pub async fn connect(database_url: &str) -> SqlitePool {
    let options = SqliteConnectOptions::from_str(database_url)
        .expect("DATABASE_URL некорректный")
        .foreign_keys(true);

    let db = SqlitePoolOptions::new()
        .max_connections(DB_MAX_CONNECTIONS)
        .connect_with(options)
        .await
        .expect("Не удалось открыть базу");

    // Миграции вшиваются в бинарник при сборке — база всегда в актуальной схеме
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Не удалось применить миграции");

    tasks::insert_seed_tasks_if_empty(&db)
        .await
        .expect("Не удалось добавить стартовые задачи");

    db
}

// Текущее время в мс — во всех таблицах время хранится так
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
