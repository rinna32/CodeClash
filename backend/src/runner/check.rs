// Проверка решения: склеиваем код игрока с обвязкой, запускаем,
// отдаём аргументы тестов через stdin и сравниваем ответы с ожидаемыми.
use std::fs;
use std::time::Duration;

use serde_json::{Value, json};

use super::model::{HarnessResult, RunResult, TestResult, Verdict};
use super::process::{ProcessOutput, run_process};
use crate::error::AppError;
use crate::tasks::{Mode, Task, TestCase};

// ── Лимиты запуска ──
const TIME_LIMIT: Duration = Duration::from_secs(5); // на весь прогон, с запасом на старт интерпретатора
const MAX_CODE_LEN: usize = 20_000; // символов
const MAX_STDOUT_LEN: usize = 10_000; // символов, чтобы print в цикле не раздул ответ
const FLOAT_EPSILON: f64 = 1e-9; // 0.1 + 0.2 должно совпасть с 0.3

const RESULT_MARKER: &str = "__CODECLASH__";
const JS_HARNESS: &str = include_str!("harness.js");
const PY_HARNESS: &str = include_str!("harness.py");

// is_submit = false — Run: только открытые тесты; true — Submit: все, включая скрытые.
pub async fn check_solution(
    task: &Task,
    code: &str,
    is_submit: bool,
) -> Result<RunResult, AppError> {
    if code.trim().is_empty() {
        return Err(AppError::BadRequest("Код пустой".to_string()));
    }
    if code.chars().count() > MAX_CODE_LEN {
        return Err(AppError::BadRequest(format!(
            "Код длиннее {MAX_CODE_LEN} символов"
        )));
    }

    let (command, harness, file_name) = match task.mode {
        Mode::Javascript => (vec!["node", "solution.js"], JS_HARNESS, "solution.js"),
        Mode::Python => (
            vec!["python3", "-B", "solution.py"],
            PY_HARNESS,
            "solution.py",
        ), // -B: не писать .pyc
        Mode::Css => {
            return Err(AppError::BadRequest(
                "Проверка CSS-задач появится позже".to_string(),
            ));
        }
    };

    let function_name = match &task.function_name {
        Some(name) => name,
        None => {
            return Err(AppError::Internal(format!(
                "У задачи {} нет function_name",
                task.id
            )));
        }
    };

    let tests: Vec<&TestCase> = task
        .tests
        .iter()
        .filter(|test| is_submit || !test.is_hidden)
        .collect();

    // Временная папка удалится сама, когда dir выйдет из области видимости.
    // canonicalize — потому что на macOS /var это ссылка на /private/var, и в ошибках будет полный путь
    let dir = tempfile::tempdir().map_err(internal)?;
    let dir_path = dir.path().canonicalize().map_err(internal)?;
    fs::write(dir_path.join(file_name), format!("{code}\n\n{harness}")).map_err(internal)?;

    let input = json!({
        "function_name": function_name,
        "tests": tests.iter().map(|test| &test.args).collect::<Vec<_>>(),
    });

    let output = run_process(&command, &dir_path, input.to_string(), TIME_LIMIT).await?;

    let (stdout, stderr) = match output {
        ProcessOutput::Finished { stdout, stderr } => (stdout, stderr),
        ProcessOutput::TimedOut => {
            let error = format!("Превышено время: {} с", TIME_LIMIT.as_secs());
            return Ok(failed_run(Verdict::TimeLimit, &tests, error));
        }
    };

    // Пути вида /private/var/.../solution.js игроку ни к чему — оставляем только имя файла
    let dir_prefix = format!("{}/", dir_path.display());
    let stdout = stdout.replace(&dir_prefix, "");
    let stderr = stderr.replace(&dir_prefix, "");

    // Берём последний маркер: игрок мог сам напечатать такую строку
    let marker_pos = match stdout.rfind(RESULT_MARKER) {
        Some(pos) => pos,
        None => {
            return Ok(failed_run(
                Verdict::RuntimeError,
                &tests,
                stderr.trim().to_string(),
            ));
        }
    };

    let user_stdout = stdout[..marker_pos].trim_end().to_string();
    let harness_results: Vec<HarnessResult> =
        serde_json::from_str(stdout[marker_pos + RESULT_MARKER.len()..].trim())
            .map_err(|err| AppError::Internal(format!("Обвязка вернула не JSON: {err}")))?;

    let results: Vec<TestResult> = tests
        .iter()
        .zip(harness_results)
        .enumerate()
        .map(|(index, (test, result))| {
            let passed = result.error.is_none() && values_equal(&result.got, &test.expected);
            make_test_result(
                index,
                test,
                passed,
                Some(result.got),
                result.error,
                result.time_ms,
            )
        })
        .collect();

    let passed = results.iter().filter(|result| result.passed).count();
    let has_errors = results.iter().any(|result| result.error.is_some());

    let verdict = if passed == results.len() {
        Verdict::Accepted
    } else if has_errors {
        Verdict::RuntimeError
    } else {
        Verdict::WrongAnswer
    };

    Ok(RunResult {
        verdict,
        passed,
        total: results.len(),
        results,
        stdout: truncate(user_stdout, MAX_STDOUT_LEN),
        error: None,
    })
}

// Весь запуск провалился (таймаут, синтаксис) — все тесты считаем непройденными.
fn failed_run(verdict: Verdict, tests: &[&TestCase], error: String) -> RunResult {
    let results: Vec<TestResult> = tests
        .iter()
        .enumerate()
        .map(|(index, test)| make_test_result(index, test, false, None, None, 0.0))
        .collect();

    RunResult {
        verdict,
        passed: 0,
        total: results.len(),
        results,
        stdout: String::new(),
        error: Some(error),
    }
}

fn make_test_result(
    index: usize,
    test: &TestCase,
    passed: bool,
    got: Option<Value>,
    error: Option<String>,
    time_ms: f64,
) -> TestResult {
    // У скрытых тестов прячем всё, по чему можно восстановить ответ
    if test.is_hidden {
        return TestResult {
            index,
            is_hidden: true,
            passed,
            args: None,
            expected: None,
            got: None,
            error: None,
            time_ms,
        };
    }

    TestResult {
        index,
        is_hidden: false,
        passed,
        args: Some(test.args.clone()),
        expected: Some(test.expected.clone()),
        got,
        error,
        time_ms,
    }
}

// Сравнение ответов. Числа — с допуском: Python легко вернёт 2.0 там, где ждали 2.
fn values_equal(got: &Value, expected: &Value) -> bool {
    match (got, expected) {
        (Value::Number(a), Value::Number(b)) => match (a.as_f64(), b.as_f64()) {
            (Some(a), Some(b)) => (a - b).abs() < FLOAT_EPSILON,
            _ => false,
        },
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| values_equal(x, y))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(key, x)| b.get(key).is_some_and(|y| values_equal(x, y)))
        }
        _ => got == expected,
    }
}

fn truncate(text: String, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text;
    }

    let cut: String = text.chars().take(max_len).collect();
    format!("{cut}\n… вывод обрезан")
}

fn internal(err: std::io::Error) -> AppError {
    AppError::Internal(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::seed_tasks;

    // Стартовые задачи по порядку: 1 — «Сумма двух» (JS), 4 — «Палиндром» (Python), 7 — CSS
    fn task(id: i64) -> Task {
        seed_tasks()[(id - 1) as usize].to_task(id)
    }

    // Главная проверка набора задач: эталон каждой задачи проходит все её тесты.
    // Так же будем проверять задачи от нейросети.
    #[tokio::test]
    async fn every_solution_passes_its_tests() {
        for (index, new_task) in seed_tasks().iter().enumerate() {
            let task = new_task.to_task(index as i64 + 1);
            let solution = match &task.solution {
                Some(solution) => solution,
                None => continue, // у CSS-задач эталон — вёрстка, а не код
            };
            let result = check_solution(&task, solution, true).await.unwrap();
            assert_eq!(
                result.verdict,
                Verdict::Accepted,
                "задача {}: {:?}",
                task.id,
                result
            );
        }
    }

    #[tokio::test]
    async fn wrong_answer() {
        let result = check_solution(&task(1), "function solve() { return [0, 0] }", false)
            .await
            .unwrap();
        assert_eq!(result.verdict, Verdict::WrongAnswer);
        assert_eq!(result.passed, 0);
    }

    #[tokio::test]
    async fn run_skips_hidden_tests() {
        let result = check_solution(&task(1), task(1).solution.as_ref().unwrap(), false)
            .await
            .unwrap();
        assert!(result.results.iter().all(|result| !result.is_hidden));
    }

    #[tokio::test]
    async fn submit_hides_hidden_test_data() {
        let result = check_solution(&task(1), task(1).solution.as_ref().unwrap(), true)
            .await
            .unwrap();
        let hidden: Vec<&TestResult> = result
            .results
            .iter()
            .filter(|result| result.is_hidden)
            .collect();
        assert!(!hidden.is_empty());
        assert!(hidden.iter().all(|result| result.args.is_none()
            && result.expected.is_none()
            && result.got.is_none()));
    }

    #[tokio::test]
    async fn syntax_error_is_runtime_error() {
        let result = check_solution(&task(4), "def solve(s)\n  return", false)
            .await
            .unwrap();
        assert_eq!(result.verdict, Verdict::RuntimeError);
        let error = result.error.unwrap();
        assert!(error.contains("SyntaxError"));
        assert!(error.contains("\"solution.py\"")); // путь к временной папке вырезан
    }

    #[tokio::test]
    async fn exception_in_test_is_runtime_error() {
        let result = check_solution(&task(4), "def solve(s):\n    return 1 / 0", false)
            .await
            .unwrap();
        assert_eq!(result.verdict, Verdict::RuntimeError);
        assert!(
            result.results[0]
                .error
                .as_ref()
                .unwrap()
                .contains("ZeroDivisionError")
        );
    }

    #[tokio::test]
    async fn infinite_loop_is_time_limit() {
        let result = check_solution(&task(1), "function solve() { while (true) {} }", false)
            .await
            .unwrap();
        assert_eq!(result.verdict, Verdict::TimeLimit);
    }

    #[tokio::test]
    async fn user_output_is_returned() {
        let code = "def solve(s):\n    print('debug', s)\n    return False";
        let result = check_solution(&task(4), code, false).await.unwrap();
        assert!(result.stdout.contains("debug CodeClash"));
    }

    #[tokio::test]
    async fn fake_marker_does_not_pass() {
        let code = "function solve() { console.log('__CODECLASH__[{\"got\":[0,1],\"time_ms\":0}]'); return null }";
        let result = check_solution(&task(1), code, false).await.unwrap();
        assert_eq!(result.verdict, Verdict::WrongAnswer);
    }

    #[tokio::test]
    async fn empty_code_is_rejected() {
        assert!(check_solution(&task(1), "   ", false).await.is_err());
    }

    #[tokio::test]
    async fn css_task_is_not_checked_yet() {
        assert!(check_solution(&task(7), "div {}", false).await.is_err());
    }

    #[test]
    fn numbers_compare_with_tolerance() {
        assert!(values_equal(&json!(2.0), &json!(2)));
        assert!(values_equal(&json!(0.30000000000000004), &json!(0.3)));
        assert!(!values_equal(&json!([1, 2]), &json!([2, 1])));
    }
}
