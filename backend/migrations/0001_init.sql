-- Базовая схема CodeClash. SQLite — один файл без отдельного сервера;
-- позже схема переедет в PostgreSQL почти без изменений.

CREATE TABLE users (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    username      TEXT    NOT NULL UNIQUE COLLATE NOCASE, -- «Rinna» и «rinna» — один пользователь
    password_hash TEXT    NOT NULL,                       -- argon2, сам пароль не храним
    created_at    INTEGER NOT NULL                        -- unix-время в мс
);

-- Рейтинг отдельно по каждому режиму: сильный питонист не обязательно силён в JS
CREATE TABLE ratings (
    user_id INTEGER NOT NULL REFERENCES users(id),
    mode    TEXT    NOT NULL,
    rating  INTEGER NOT NULL DEFAULT 1000,
    games   INTEGER NOT NULL DEFAULT 0,
    wins    INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, mode)
);

CREATE TABLE tasks (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    mode          TEXT    NOT NULL,
    difficulty    TEXT    NOT NULL,
    title         TEXT    NOT NULL,
    statement_md  TEXT    NOT NULL,
    function_name TEXT,                -- NULL у CSS-задач
    params        TEXT    NOT NULL DEFAULT '[]', -- имена аргументов для заготовки кода, JSON
    tests         TEXT    NOT NULL DEFAULT '[]', -- [{ args, expected, is_hidden }], JSON
    solution      TEXT,                -- эталон автора задачи
    bot_solution  TEXT,                -- решение нейросети-соперника, написанное отдельно
    bot_passed    INTEGER NOT NULL DEFAULT 0, -- сколько тестов честно прошло решение бота
    target_html   TEXT,                -- эталонная вёрстка CSS-задачи
    source        TEXT    NOT NULL DEFAULT 'manual', -- manual | llm
    created_at    INTEGER NOT NULL
);

CREATE INDEX tasks_pick_idx ON tasks (mode, difficulty);

CREATE TABLE matches (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id        INTEGER NOT NULL REFERENCES users(id),
    task_id        INTEGER NOT NULL REFERENCES tasks(id),
    mode           TEXT    NOT NULL,
    difficulty     TEXT    NOT NULL,
    bot_level      TEXT    NOT NULL,
    total_tests    INTEGER NOT NULL,
    starts_at      INTEGER NOT NULL, -- после обратного отсчёта
    ends_at        INTEGER NOT NULL,
    bot_schedule   TEXT    NOT NULL, -- [{ at, passed }] — когда бот «отправляет» решения; клиенту не отдаём
    player_passed  INTEGER NOT NULL DEFAULT 0, -- лучший результат игрока
    player_best_at INTEGER,          -- когда этот результат получен
    player_code    TEXT,             -- последний отправленный код, для истории
    status         TEXT    NOT NULL DEFAULT 'active', -- active | finished
    result         TEXT,             -- win | loss | draw
    finished_at    INTEGER,
    rating_delta   INTEGER
);

CREATE INDEX matches_user_idx ON matches (user_id, id);
