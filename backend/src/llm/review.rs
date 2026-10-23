// Разбор решения игрока: нейросеть сравнивает его код с эталоном и объясняет ошибки.
// Результат автотестов считаем сами и даём ей как факт — на глаз она может назвать
// рабочий код сломанным.
use serde::{Deserialize, Serialize};

use super::client::LlmClient;
use super::prompts;
use crate::error::AppError;
use crate::runner::{RunResult, check_solution};
use crate::tasks::{Mode, Task};

const MAX_FAILED_SHOWN: usize = 3; // сколько упавших тестов показать нейросети — хватит, чтобы понять ошибку
const MAX_CODE_LEN: usize = 20_000; // символов, как в проверке решений

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub summary: String,  // решена ли задача и главное впечатление
    pub approach: String, // идея правильного решения
    #[serde(default)]
    pub mistakes: Vec<String>,
    #[serde(default)]
    pub tips: Vec<String>,
}

pub async fn review_solution(llm: &LlmClient, task: &Task, code: &str) -> Result<Review, AppError> {
    if code.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Код пустой — разбирать нечего".to_string(),
        ));
    }
    if code.chars().count() > MAX_CODE_LEN {
        return Err(AppError::BadRequest(format!(
            "Код длиннее {MAX_CODE_LEN} символов"
        )));
    }

    let test_results = match task.mode {
        Mode::Css => "автопроверки вёрстки нет — сравни с эталоном сам".to_string(),
        _ => match check_solution(task, code, true).await {
            Ok(result) => describe(task, &result),
            Err(err) => format!("код не удалось проверить: {err}"),
        },
    };

    let value = llm
        .chat_json(
            &prompts::reviewer_system(),
            &prompts::reviewer_user(
                task.mode,
                &task.statement_md,
                &task.reference_solution(),
                code,
                &test_results,
            ),
        )
        .await?;

    serde_json::from_value(value)
        .map_err(|err| AppError::Llm(format!("разбор не по формату: {err}")))
}

// «пройдено 4 из 6, WrongAnswer; solve([1, 2]) → ждали 3, получили 2»
fn describe(task: &Task, result: &RunResult) -> String {
    let mut text = format!(
        "пройдено {} из {} тестов, вердикт {:?}",
        result.passed, result.total, result.verdict
    );

    if let Some(error) = &result.error {
        text.push_str(&format!("; ошибка запуска: {error}"));
    }

    // Игрок уже сдался или матч закончился, поэтому нейросети можно показать и скрытые тесты
    let failed = result
        .results
        .iter()
        .filter(|test| !test.passed)
        .take(MAX_FAILED_SHOWN);
    for test in failed {
        let case = match task.tests.get(test.index) {
            Some(case) => case,
            None => continue,
        };
        let args = case
            .args
            .iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let got = match (&test.got, &test.error) {
            (_, Some(error)) => format!("ошибка: {error}"),
            (Some(got), None) => format!("получили {got}"),
            (None, None) => "ответ скрыт".to_string(),
        };
        text.push_str(&format!("; solve({args}) → ждали {}, {got}", case.expected));
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::seed_tasks;

    #[actix_web::test]
    async fn describes_failed_tests_including_hidden() {
        // «Сумма двух»: решение всегда возвращает [0, 1] — проходит только первый пример
        let task = seed_tasks()[0].to_task(1);
        let result = check_solution(
            &task,
            "function solve(nums, target) { return [0, 1] }",
            true,
        )
        .await
        .unwrap();

        let text = describe(&task, &result);

        assert!(text.starts_with(&format!("пройдено 1 из {} тестов", task.tests.len())));
        assert!(text.contains("solve([3,2,4], 6) → ждали [1,2], получили [0,1]"));
        assert_eq!(text.matches("solve(").count(), MAX_FAILED_SHOWN);
    }
}
