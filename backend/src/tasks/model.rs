use serde::{Deserialize, Serialize};
use serde_json::Value;

// Режим матча. Язык и режим — одно понятие: JS и Python — алгоритмы, CSS — вёрстка.
// sqlx::Type — в базе хранится строкой 'javascript' / 'python' / 'css'
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum Mode {
    Javascript,
    Python,
    Css,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Mode {
    pub fn language_name(&self) -> &'static str {
        match self {
            Mode::Javascript => "JavaScript",
            Mode::Python => "Python",
            Mode::Css => "CSS",
        }
    }
}

// Один тест: аргументы для функции игрока и ожидаемый ответ.
// Скрытые тесты на клиент не уходят — их проверяет только Submit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub args: Vec<Value>,
    pub expected: Value,
    pub is_hidden: bool,
}

// Задача целиком из базы. Наружу не отдаём: тут скрытые тесты и решения.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Task {
    pub id: i64,
    pub mode: Mode,
    pub difficulty: Difficulty,
    pub title: String,
    pub statement_md: String,
    pub function_name: Option<String>, // None у CSS-задач: там нет функции
    #[sqlx(json)]
    pub params: Vec<String>, // имена аргументов — фронт строит из них заготовку кода
    #[sqlx(json)]
    pub tests: Vec<TestCase>,
    pub solution: Option<String>, // эталон автора задачи — игрок видит его после матча или сдавшись
    // Решение бота сервер не читает: оно проверено при создании задачи, а дальше — для админки
    #[allow(dead_code)]
    pub bot_solution: Option<String>, // решение соперника — нейросеть писала его отдельно
    pub bot_passed: i64,             // сколько тестов это решение честно прошло
    pub target_html: Option<String>, // эталонная вёрстка для CSS-режима
    pub source: String,              // manual | llm
}

impl Task {
    // Правильный ответ, который игрок видит после матча или сдавшись. У CSS-задач это готовая вёрстка
    pub fn reference_solution(&self) -> String {
        match (&self.solution, &self.target_html) {
            (Some(solution), _) => solution.clone(),
            (None, Some(target_html)) => target_html.clone(),
            (None, None) => String::new(),
        }
    }
}

// Новая задача — от нейросети или из стартового набора
pub struct NewTask {
    pub mode: Mode,
    pub difficulty: Difficulty,
    pub title: String,
    pub statement_md: String,
    pub function_name: Option<String>,
    pub params: Vec<String>,
    pub tests: Vec<TestCase>,
    pub solution: Option<String>,
    pub bot_solution: Option<String>,
    pub bot_passed: i64,
    pub target_html: Option<String>,
    pub source: &'static str,
}

impl NewTask {
    // Временная задача без записи в базу — чтобы прогнать решения через раннер до сохранения
    pub fn to_task(&self, id: i64) -> Task {
        Task {
            id,
            mode: self.mode,
            difficulty: self.difficulty,
            title: self.title.clone(),
            statement_md: self.statement_md.clone(),
            function_name: self.function_name.clone(),
            params: self.params.clone(),
            tests: self.tests.clone(),
            solution: self.solution.clone(),
            bot_solution: self.bot_solution.clone(),
            bot_passed: self.bot_passed,
            target_html: self.target_html.clone(),
            source: self.source.to_string(),
        }
    }
}

// ── Что видит клиент ──

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TaskListItem {
    pub id: i64,
    pub mode: Mode,
    pub difficulty: Difficulty,
    pub title: String,
    pub source: String,
}

#[derive(Debug, Serialize)]
pub struct TaskDetail {
    pub id: i64,
    pub mode: Mode,
    pub difficulty: Difficulty,
    pub title: String,
    pub statement_md: String,
    pub function_name: Option<String>,
    pub params: Vec<String>,
    pub examples: Vec<TestCase>, // только открытые тесты
    pub hidden_tests_count: usize,
    pub target_html: Option<String>,
    pub source: String,
}

impl From<&Task> for TaskDetail {
    fn from(task: &Task) -> TaskDetail {
        let examples: Vec<TestCase> = task
            .tests
            .iter()
            .filter(|test| !test.is_hidden)
            .cloned()
            .collect();
        let hidden_tests_count = task.tests.iter().filter(|test| test.is_hidden).count();

        TaskDetail {
            id: task.id,
            mode: task.mode,
            difficulty: task.difficulty,
            title: task.title.clone(),
            statement_md: task.statement_md.clone(),
            function_name: task.function_name.clone(),
            params: task.params.clone(),
            examples,
            hidden_tests_count,
            target_html: task.target_html.clone(),
            source: task.source.clone(),
        }
    }
}

// Фильтр для GET /api/tasks?mode=python&difficulty=easy — оба поля необязательные.
#[derive(Debug, Deserialize)]
pub struct TaskFilter {
    pub mode: Option<Mode>,
    pub difficulty: Option<Difficulty>,
}
