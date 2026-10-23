// Генерация задачи нейросетью:
// 1) автор пишет задачу, тесты и эталон → 2) эталон прогоняем по тестам: не прошёл примеры — в мусор,
//    не прошёл скрытый тест — выкидываем этот тест
// → 3) нейросеть отдельно решает задачу как соперник → 4) решение бота прогоняем по тем же тестам.
use serde::Deserialize;
use serde_json::Value;

use super::client::LlmClient;
use super::prompts;
use crate::error::AppError;
use crate::runner::{Verdict, check_solution};
use crate::tasks::{Difficulty, Mode, NewTask, TestCase};

const MAX_AUTHOR_ATTEMPTS: usize = 2; // бесплатные лимиты маленькие — больше двух попыток не тратим
const MIN_HIDDEN_TESTS: usize = 2;

// Что присылает автор задачи
#[derive(Debug, Deserialize)]
struct AuthoredTask {
    title: String,
    statement_md: String,
    params: Vec<String>,
    examples: Vec<AuthoredTest>,
    hidden_tests: Vec<AuthoredTest>,
    solution: String,
}

#[derive(Debug, Deserialize)]
struct AuthoredTest {
    args: Vec<Value>,
    expected: Value,
}

#[derive(Debug, Deserialize)]
struct BotAnswer {
    code: String,
}

pub async fn generate_task(
    llm: &LlmClient,
    mode: Mode,
    difficulty: Difficulty,
) -> Result<NewTask, AppError> {
    let mut last_error = AppError::Llm("нейросеть не ответила".to_string());

    for attempt in 1..=MAX_AUTHOR_ATTEMPTS {
        match author_task(llm, mode, difficulty).await {
            Ok(task) => return solve_as_bot(llm, task).await,
            Err(err) => {
                log::warn!("Генерация, попытка {attempt}: {err}");
                last_error = err;
            }
        }
    }

    Err(last_error)
}

// Шаги 1–2: задача от нейросети, проверенная её же эталоном
async fn author_task(
    llm: &LlmClient,
    mode: Mode,
    difficulty: Difficulty,
) -> Result<NewTask, AppError> {
    let value = llm
        .chat_json(
            &prompts::author_system(),
            &prompts::author_user(mode, difficulty),
        )
        .await?;
    let authored: AuthoredTask = serde_json::from_value(value)
        .map_err(|err| AppError::Llm(format!("задача не по формату: {err}")))?;

    if authored.examples.is_empty() || authored.hidden_tests.len() < MIN_HIDDEN_TESTS {
        return Err(AppError::Llm("слишком мало тестов".to_string()));
    }
    if authored.title.trim().is_empty() || authored.solution.trim().is_empty() {
        return Err(AppError::Llm("пустое название или решение".to_string()));
    }

    let to_tests = |tests: Vec<AuthoredTest>, is_hidden: bool| -> Vec<TestCase> {
        tests
            .into_iter()
            .map(|test| TestCase {
                args: test.args,
                expected: test.expected,
                is_hidden,
            })
            .collect()
    };
    let mut tests = to_tests(authored.examples, false);
    tests.extend(to_tests(authored.hidden_tests, true));

    let mut task = NewTask {
        mode,
        difficulty,
        title: authored.title.trim().to_string(),
        statement_md: authored.statement_md.trim().to_string(),
        function_name: Some("solve".to_string()),
        params: authored.params,
        tests,
        solution: Some(authored.solution),
        bot_solution: None,
        bot_passed: 0,
        target_html: None,
        source: "llm",
    };

    drop_wrong_hidden_tests(&mut task).await?;
    Ok(task)
}

// Нейросеть часто ошибается в ожидаемых ответах — эталон прогоняем по всем тестам.
// Открытые примеры игрок видит в условии, поэтому эталон обязан пройти их все.
// Скрытый тест, на котором эталон «ошибся», скорее всего посчитан неверно — выкидываем его,
// если скрытых останется достаточно. Иначе задача в мусор
async fn drop_wrong_hidden_tests(task: &mut NewTask) -> Result<(), AppError> {
    let check = check_solution(
        &task.to_task(0),
        task.solution.as_deref().unwrap_or(""),
        true,
    )
    .await?;
    if check.verdict == Verdict::Accepted {
        return Ok(());
    }

    let reject = AppError::Llm(format!(
        "эталон не прошёл свои тесты: {}/{} ({:?})",
        check.passed, check.total, check.verdict
    ));

    // Падение или таймаут — проблема в самом эталоне, а не в тестах
    if check.verdict != Verdict::WrongAnswer {
        return Err(reject);
    }

    let failed_open = check
        .results
        .iter()
        .any(|result| !result.passed && !result.is_hidden);
    if failed_open {
        return Err(reject);
    }

    let failed: Vec<usize> = check
        .results
        .iter()
        .filter(|result| !result.passed)
        .map(|result| result.index)
        .collect();
    let hidden_left = task
        .tests
        .iter()
        .enumerate()
        .filter(|(index, test)| test.is_hidden && !failed.contains(index))
        .count();
    if hidden_left < MIN_HIDDEN_TESTS {
        return Err(reject);
    }

    log::info!(
        "Эталон не согласен со скрытыми тестами ({} шт.) — выкидываем их",
        failed.len()
    );
    let mut index = 0;
    task.tests.retain(|_| {
        let keep = !failed.contains(&index);
        index += 1;
        keep
    });

    Ok(())
}

// Шаги 3–4: нейросеть решает задачу «с нуля», видя только условие и примеры — как игрок
async fn solve_as_bot(llm: &LlmClient, mut task: NewTask) -> Result<NewTask, AppError> {
    let signature = signature(&task);
    let examples = task
        .tests
        .iter()
        .filter(|test| !test.is_hidden)
        .map(|test| format!("solve({}) → {}", join_args(&test.args), test.expected))
        .collect::<Vec<_>>()
        .join("\n");

    let value = llm
        .chat_json(
            &prompts::solver_system(),
            &prompts::solver_user(task.mode, &task.statement_md, &signature, &examples),
        )
        .await?;
    let answer: BotAnswer = serde_json::from_value(value)
        .map_err(|err| AppError::Llm(format!("решение бота не по формату: {err}")))?;

    // Честный результат: сколько тестов на самом деле прошло решение бота. Может быть и не все
    let check = check_solution(&task.to_task(0), &answer.code, true).await;
    task.bot_passed = match check {
        Ok(result) => result.passed as i64,
        Err(_) => 0, // например, бот прислал пустой код
    };
    task.bot_solution = Some(answer.code);

    Ok(task)
}

fn signature(task: &NewTask) -> String {
    let params = task.params.join(", ");
    match task.mode {
        Mode::Python => format!("def solve({params}):"),
        _ => format!("function solve({params})"),
    }
}

fn join_args(args: &[Value]) -> String {
    args.iter()
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use actix_web::{App, HttpResponse, HttpServer, post, web};
    use serde_json::json;

    use super::*;
    use crate::config::LlmConfig;

    // Поддельная нейросеть в формате OpenAI: автору отдаём задачу, «сопернику» — решение
    #[post("/chat/completions")]
    async fn fake_llm(body: web::Json<Value>, answers: web::Data<(Value, Value)>) -> HttpResponse {
        let system = body["messages"][0]["content"].as_str().unwrap_or_default();
        let answer = if system.contains("автор") {
            &answers.0
        } else {
            &answers.1
        };

        HttpResponse::Ok().json(json!({
            "choices": [{ "message": { "content": format!("```json\n{answer}\n```") } }]
        }))
    }

    async fn start_fake_llm(author: Value, solver: Value) -> LlmClient {
        let answers = web::Data::new((author, solver));
        let server =
            HttpServer::new(move || App::new().app_data(answers.clone()).service(fake_llm))
                .bind(("127.0.0.1", 0))
                .unwrap();
        let port = server.addrs()[0].port();
        actix_web::rt::spawn(server.run());

        LlmClient::new(LlmConfig {
            base_url: format!("http://127.0.0.1:{port}"),
            api_key: String::new(),
            model: "fake".to_string(),
        })
    }

    fn authored(solution: &str) -> Value {
        authored_with_hidden(
            solution,
            json!([{ "args": [[]], "expected": 0 }, { "args": [[-1, 1]], "expected": 0 }]),
        )
    }

    fn authored_with_hidden(solution: &str, hidden_tests: Value) -> Value {
        json!({
            "title": "Сумма массива",
            "statement_md": "Верни сумму чисел массива `nums`.",
            "params": ["nums"],
            "examples": [{ "args": [[1, 2, 3]], "expected": 6 }, { "args": [[5]], "expected": 5 }],
            "hidden_tests": hidden_tests,
            "solution": solution,
        })
    }

    const SUM_SOLUTION: &str = "function solve(nums) { return nums.reduce((a, b) => a + b, 0) }";

    #[actix_web::test]
    async fn generates_task_and_bot_solution() {
        let llm = start_fake_llm(
            authored(SUM_SOLUTION),
            // Бот ошибся: на пустом массиве reduce без начального значения падает
            json!({ "code": "function solve(nums) { return nums.reduce((a, b) => a + b) }" }),
        )
        .await;

        let task = generate_task(&llm, Mode::Javascript, Difficulty::Easy)
            .await
            .unwrap();

        assert_eq!(task.title, "Сумма массива");
        assert_eq!(task.tests.len(), 4);
        assert_eq!(task.tests.iter().filter(|test| test.is_hidden).count(), 2);
        assert_eq!(task.bot_passed, 3); // честно: бот прошёл 3 из 4
        assert!(task.bot_solution.is_some());
    }

    #[actix_web::test]
    async fn rejects_task_whose_solution_fails_its_tests() {
        let llm = start_fake_llm(
            authored("function solve(nums) { return 42 }"),
            json!({ "code": "function solve() {}" }),
        )
        .await;

        let result = generate_task(&llm, Mode::Javascript, Difficulty::Easy).await;
        assert!(matches!(result, Err(AppError::Llm(message)) if message.contains("эталон")));
    }

    #[actix_web::test]
    async fn drops_hidden_test_with_wrong_expected() {
        let llm = start_fake_llm(
            authored_with_hidden(
                SUM_SOLUTION,
                // Нейросеть ошиблась в ответе последнего теста: 2 + 2 ≠ 5
                json!([
                    { "args": [[]], "expected": 0 },
                    { "args": [[-1, 1]], "expected": 0 },
                    { "args": [[2, 2]], "expected": 5 },
                ]),
            ),
            json!({ "code": SUM_SOLUTION }),
        )
        .await;

        let task = generate_task(&llm, Mode::Javascript, Difficulty::Easy)
            .await
            .unwrap();

        assert_eq!(task.tests.len(), 4);
        assert!(
            task.tests
                .iter()
                .all(|test| test.args != vec![json!([2, 2])])
        );
        assert_eq!(task.bot_passed, 4);
    }
}
