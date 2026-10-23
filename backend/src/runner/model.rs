use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Accepted,     // все тесты пройдены
    WrongAnswer,  // код отработал, но ответ не тот
    RuntimeError, // исключение или синтаксическая ошибка
    TimeLimit,    // не уложился во время
}

// Результат одного теста. У скрытых тестов не показываем аргументы и ответы —
// иначе их можно подсмотреть и захардкодить.
#[derive(Debug, Serialize)]
pub struct TestResult {
    pub index: usize,
    pub is_hidden: bool,
    pub passed: bool,
    pub args: Option<Vec<Value>>,
    pub expected: Option<Value>,
    pub got: Option<Value>,
    pub error: Option<String>,
    pub time_ms: f64,
}

#[derive(Debug, Serialize)]
pub struct RunResult {
    pub verdict: Verdict,
    pub passed: usize,
    pub total: usize,
    pub results: Vec<TestResult>,
    pub stdout: String,        // всё, что игрок вывел через console.log / print
    pub error: Option<String>, // ошибка всего запуска: синтаксис, таймаут
}

// То, что печатает обвязка про каждый тест. Сравнение с ответом — уже в Rust.
#[derive(Debug, Deserialize)]
pub struct HarnessResult {
    pub got: Value,
    pub error: Option<String>,
    pub time_ms: f64,
}
