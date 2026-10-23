// Бот-соперник. Код он не пишет вживую: нейросеть решила задачу заранее,
// а здесь мы строим расписание, когда бот «отправит» свои попытки.
// Больше тестов, чем честно прошло его решение, бот набрать не может.
use super::model::{BotLevel, BotStep};

struct BotProfile {
    finish_from: f64,    // доля времени матча, раньше которой бот не дорешает
    finish_to: f64,      // ... и позже которой не затянет
    steps: i64,          // сколько промежуточных отправок
    give_up_chance: f64, // шанс застрять на половине тестов
}

fn profile(level: BotLevel) -> BotProfile {
    match level {
        BotLevel::Novice => BotProfile {
            finish_from: 0.6,
            finish_to: 0.95,
            steps: 3,
            give_up_chance: 0.35,
        },
        BotLevel::Middle => BotProfile {
            finish_from: 0.35,
            finish_to: 0.65,
            steps: 2,
            give_up_chance: 0.15,
        },
        BotLevel::Senior => BotProfile {
            finish_from: 0.12,
            finish_to: 0.35,
            steps: 1,
            give_up_chance: 0.0,
        },
    }
}

pub fn bot_rating(level: BotLevel) -> i64 {
    match level {
        BotLevel::Novice => 900,
        BotLevel::Middle => 1200,
        BotLevel::Senior => 1500,
    }
}

// seed — любое число (время + id игрока): расписание случайное, но без внешних крейтов
pub fn build_schedule(
    level: BotLevel,
    bot_passed: i64,
    total_tests: i64,
    starts_at: i64,
    duration_ms: i64,
    seed: u64,
) -> Vec<BotStep> {
    let profile = profile(level);
    let mut random = Random::new(seed);

    let mut final_passed = bot_passed.min(total_tests);
    if random.next() < profile.give_up_chance {
        final_passed = final_passed.min(total_tests / 2);
    }
    if final_passed == 0 {
        return vec![]; // бот так ничего и не отправит
    }

    let finish_share =
        profile.finish_from + random.next() * (profile.finish_to - profile.finish_from);
    let finish_at = (duration_ms as f64 * finish_share) as i64;
    let steps = profile.steps.min(final_passed);

    (1..=steps)
        .map(|step| BotStep {
            at: starts_at + finish_at * step / steps,
            passed: (final_passed * step + steps - 1) / steps, // округление вверх: на последнем шаге — все
        })
        .collect()
}

// Сколько тестов бот прошёл к моменту time и когда впервые набрал это число
pub fn progress_at(schedule: &[BotStep], time: i64) -> (i64, Option<i64>) {
    match schedule.iter().rev().find(|step| step.at <= time) {
        Some(step) => (step.passed, Some(step.at)),
        None => (0, None),
    }
}

// xorshift — простой генератор псевдослучайных чисел, нам хватит
struct Random(u64);

impl Random {
    fn new(seed: u64) -> Random {
        Random(seed.max(1))
    }

    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % 10_000) as f64 / 10_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINUTE: i64 = 60_000;

    #[test]
    fn bot_never_passes_more_than_its_solution() {
        for seed in 1..200 {
            let schedule = build_schedule(BotLevel::Senior, 3, 5, 0, 5 * MINUTE, seed);
            assert!(schedule.iter().all(|step| step.passed <= 3));
        }
    }

    #[test]
    fn senior_finishes_within_its_window() {
        for seed in 1..200 {
            let schedule = build_schedule(BotLevel::Senior, 4, 4, 0, 10 * MINUTE, seed);
            let last = schedule.last().unwrap();
            assert_eq!(last.passed, 4);
            assert!(last.at >= MINUTE && last.at <= 4 * MINUTE);
        }
    }

    #[test]
    fn progress_grows_over_time() {
        let schedule = vec![
            BotStep { at: 100, passed: 2 },
            BotStep { at: 200, passed: 4 },
        ];
        assert_eq!(progress_at(&schedule, 50), (0, None));
        assert_eq!(progress_at(&schedule, 150), (2, Some(100)));
        assert_eq!(progress_at(&schedule, 999), (4, Some(200)));
    }
}
