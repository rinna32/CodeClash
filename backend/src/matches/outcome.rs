// Правила победы. Считаем «лениво» — при каждом запросе к матчу:
// 1) кто первым прошёл все тесты — побеждает сразу;
// 2) по таймеру — у кого больше тестов; 3) при равенстве — кто раньше набрал свой результат;
// 4) никто ничего не отправил — ничья.
use super::bot::progress_at;
use super::model::{MatchResult, MatchRow};

pub fn decide(row: &MatchRow, now: i64) -> Option<(MatchResult, i64)> {
    let total = row.total_tests;

    let bot_full_at = row
        .bot_schedule
        .iter()
        .find(|step| step.passed == total)
        .map(|step| step.at);
    let player_full_at = if row.player_passed == total {
        row.player_best_at
    } else {
        None
    };

    // Бот дорешал раньше игрока (и это время уже наступило)
    if let Some(bot_at) = bot_full_at {
        let is_bot_first = player_full_at.is_none_or(|player_at| bot_at < player_at);
        if bot_at <= now.min(row.ends_at) && is_bot_first {
            return Some((MatchResult::Loss, bot_at));
        }
    }

    if let Some(player_at) = player_full_at {
        return Some((MatchResult::Win, player_at));
    }

    if now < row.ends_at {
        return None; // матч идёт
    }

    // Время вышло — сравниваем лучшие результаты
    let (bot_passed, bot_best_at) = progress_at(&row.bot_schedule, row.ends_at);
    let player_passed = row.player_passed;

    let result = if player_passed > bot_passed {
        MatchResult::Win
    } else if player_passed < bot_passed {
        MatchResult::Loss
    } else {
        match (row.player_best_at, bot_best_at) {
            (Some(player_at), Some(bot_at)) if player_at < bot_at => MatchResult::Win,
            (Some(player_at), Some(bot_at)) if player_at > bot_at => MatchResult::Loss,
            _ => MatchResult::Draw, // оба по нулям или одновременно
        }
    };

    Some((result, row.ends_at))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matches::model::{BotLevel, BotStep};
    use crate::tasks::{Difficulty, Mode};

    fn row(schedule: Vec<BotStep>, player_passed: i64, player_best_at: Option<i64>) -> MatchRow {
        MatchRow {
            id: 1,
            user_id: 1,
            task_id: 1,
            mode: Mode::Javascript,
            difficulty: Difficulty::Easy,
            bot_level: BotLevel::Middle,
            total_tests: 4,
            starts_at: 0,
            ends_at: 1000,
            bot_schedule: schedule,
            player_passed,
            player_best_at,
            status: "active".to_string(),
            result: None,
            finished_at: None,
            rating_delta: None,
        }
    }

    #[test]
    fn player_solves_first() {
        let match_row = row(vec![BotStep { at: 500, passed: 4 }], 4, Some(300));
        assert_eq!(decide(&match_row, 400), Some((MatchResult::Win, 300)));
    }

    #[test]
    fn bot_solves_first() {
        let match_row = row(vec![BotStep { at: 500, passed: 4 }], 2, Some(300));
        assert_eq!(decide(&match_row, 400), None); // бот ещё не отправил
        assert_eq!(decide(&match_row, 600), Some((MatchResult::Loss, 500)));
    }

    #[test]
    fn timeout_more_tests_wins() {
        let match_row = row(vec![BotStep { at: 500, passed: 2 }], 3, Some(900));
        assert_eq!(decide(&match_row, 1500), Some((MatchResult::Win, 1000)));
    }

    #[test]
    fn timeout_tie_earlier_wins() {
        let match_row = row(vec![BotStep { at: 500, passed: 2 }], 2, Some(800));
        assert_eq!(decide(&match_row, 1500), Some((MatchResult::Loss, 1000)));
    }

    #[test]
    fn nobody_submitted_is_draw() {
        let match_row = row(vec![], 0, None);
        assert_eq!(decide(&match_row, 1500), Some((MatchResult::Draw, 1000)));
    }
}
