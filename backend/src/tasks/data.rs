// Стартовые задачи, написанные вручную. Их дополняет генерация через нейросеть,
// но этот набор остаётся страховкой: матч можно начать, даже если нейросеть недоступна.
use serde_json::{Value, json};
use sqlx::SqlitePool;

use super::model::{Difficulty, Mode, NewTask, TestCase};
use super::repo;
use crate::error::AppError;

pub async fn insert_seed_tasks_if_empty(db: &SqlitePool) -> Result<(), AppError> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tasks")
        .fetch_one(db)
        .await?;
    if count > 0 {
        return Ok(());
    }

    for task in seed_tasks() {
        repo::insert(db, &task).await?;
    }
    log::info!("Добавлены стартовые задачи");

    Ok(())
}

// Короткая запись теста, чтобы список задач читался как таблица
fn test(args: Value, expected: Value, is_hidden: bool) -> TestCase {
    TestCase {
        args: args.as_array().cloned().unwrap_or_default(),
        expected,
        is_hidden,
    }
}

// У стартовых задач бот играет эталоном — он решает их полностью
fn algo_task(
    mode: Mode,
    difficulty: Difficulty,
    title: &str,
    statement_md: &str,
    params: &[&str],
    tests: Vec<TestCase>,
    solution: &str,
) -> NewTask {
    NewTask {
        mode,
        difficulty,
        title: title.to_string(),
        statement_md: statement_md.to_string(),
        function_name: Some("solve".to_string()),
        params: params.iter().map(|param| param.to_string()).collect(),
        bot_passed: tests.len() as i64,
        tests,
        solution: Some(solution.to_string()),
        bot_solution: Some(solution.to_string()),
        target_html: None,
        source: "manual",
    }
}

pub fn seed_tasks() -> Vec<NewTask> {
    vec![
        // ───────────────────────── JavaScript ─────────────────────────
        algo_task(
            Mode::Javascript,
            Difficulty::Easy,
            "Сумма двух",
            "Дан массив `nums` и число `target`. Верни индексы двух элементов, сумма которых равна `target`.",
            &["nums", "target"],
            vec![
                test(json!([[2, 7, 11, 15], 9]), json!([0, 1]), false),
                test(json!([[3, 2, 4], 6]), json!([1, 2]), false),
                test(json!([[1, 5, 3, 8], 11]), json!([2, 3]), true),
                test(json!([[0, -4, 4], 0]), json!([1, 2]), true),
            ],
            "function solve(nums, target) {
  const seen = new Map()
  for (let i = 0; i < nums.length; i++) {
    if (seen.has(target - nums[i])) return [seen.get(target - nums[i]), i]
    seen.set(nums[i], i)
  }
}",
        ),
        algo_task(
            Mode::Javascript,
            Difficulty::Medium,
            "Сжатие строки",
            "Сожми строку: подряд идущие одинаковые символы замени на символ и их количество. `\"aaabcc\"` → `\"a3b1c2\"`.",
            &["s"],
            vec![
                test(json!(["aaabcc"]), json!("a3b1c2"), false),
                test(json!(["x"]), json!("x1"), false),
                test(json!([""]), json!(""), true),
                test(json!(["zzzzzzzzzzzz"]), json!("z12"), true),
            ],
            "function solve(s) {
  let result = ''
  let i = 0
  while (i < s.length) {
    let j = i
    while (s[j] === s[i]) j++
    result += s[i] + (j - i)
    i = j
  }
  return result
}",
        ),
        algo_task(
            Mode::Javascript,
            Difficulty::Hard,
            "Подстрока без повторов",
            "Верни длину самой длинной подстроки строки `s`, в которой нет повторяющихся символов.",
            &["s"],
            vec![
                test(json!(["abcabcbb"]), json!(3), false),
                test(json!(["bbbbb"]), json!(1), false),
                test(json!(["pwwkew"]), json!(3), true),
                test(json!([""]), json!(0), true),
                test(json!(["dvdf"]), json!(3), true),
            ],
            "function solve(s) {
  const last = new Map()
  let start = 0
  let best = 0
  for (let i = 0; i < s.length; i++) {
    if (last.has(s[i]) && last.get(s[i]) >= start) start = last.get(s[i]) + 1
    last.set(s[i], i)
    best = Math.max(best, i - start + 1)
  }
  return best
}",
        ),
        // ───────────────────────── Python ─────────────────────────
        algo_task(
            Mode::Python,
            Difficulty::Easy,
            "Палиндром",
            "Верни `True`, если строка `s` читается одинаково слева направо и справа налево. Регистр и пробелы не учитываются.",
            &["s"],
            vec![
                test(json!(["А роза упала на лапу Азора"]), json!(true), false),
                test(json!(["CodeClash"]), json!(false), false),
                test(json!([""]), json!(true), true),
                test(json!(["Ab ba"]), json!(true), true),
            ],
            "def solve(s):
    cleaned = s.replace(' ', '').lower()
    return cleaned == cleaned[::-1]",
        ),
        algo_task(
            Mode::Python,
            Difficulty::Medium,
            "Скобки",
            "Строка состоит из скобок `()[]{}`. Верни `True`, если все скобки закрыты в правильном порядке.",
            &["s"],
            vec![
                test(json!(["()[]{}"]), json!(true), false),
                test(json!(["([)]"]), json!(false), false),
                test(json!(["{[]}"]), json!(true), true),
                test(json!(["(("]), json!(false), true),
            ],
            "def solve(s):
    pairs = {')': '(', ']': '[', '}': '{'}
    stack = []
    for ch in s:
        if ch in pairs:
            if not stack or stack.pop() != pairs[ch]:
                return False
        else:
            stack.append(ch)
    return not stack",
        ),
        algo_task(
            Mode::Python,
            Difficulty::Hard,
            "Слияние интервалов",
            "Дан список интервалов `intervals` вида `[начало, конец]`. Объедини пересекающиеся и верни результат, отсортированный по началу.",
            &["intervals"],
            vec![
                test(json!([[[1, 3], [2, 6], [8, 10]]]), json!([[1, 6], [8, 10]]), false),
                test(json!([[[1, 4], [4, 5]]]), json!([[1, 5]]), false),
                test(json!([[[5, 7], [1, 2]]]), json!([[1, 2], [5, 7]]), true),
                test(json!([[]]), json!([]), true),
                test(json!([[[1, 10], [2, 3], [4, 5]]]), json!([[1, 10]]), true),
            ],
            "def solve(intervals):
    result = []
    for start, end in sorted(intervals):
        if result and start <= result[-1][1]:
            result[-1][1] = max(result[-1][1], end)
        else:
            result.append([start, end])
    return result",
        ),
        // ───────────────────────── CSS ─────────────────────────
        NewTask {
            mode: Mode::Css,
            difficulty: Difficulty::Easy,
            title: "Квадрат в центре".to_string(),
            statement_md: "Нарисуй чёрный квадрат 100×100 точно по центру белого холста 400×300.".to_string(),
            function_name: None,
            params: vec![],
            tests: vec![],
            solution: None,
            bot_solution: None,
            bot_passed: 0,
            target_html: Some(
                "<div></div>
<style>
  body { margin: 0; width: 400px; height: 300px; background: #fff; display: grid; place-items: center; }
  div { width: 100px; height: 100px; background: #000; }
</style>"
                    .to_string(),
            ),
            source: "manual",
        },
    ]
}
