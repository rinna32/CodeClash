// Рейтинг Эло и лиги. Рейтинг отдельный для каждого режима.

pub const START_RATING: i64 = 1000;
const K_FACTOR: f64 = 32.0; // насколько сильно меняется рейтинг за одну игру

// score: 1 — победа, 0.5 — ничья, 0 — поражение
pub fn rating_delta(player_rating: i64, bot_rating: i64, score: f64) -> i64 {
    let expected = 1.0 / (1.0 + 10f64.powf((bot_rating - player_rating) as f64 / 400.0));
    (K_FACTOR * (score - expected)).round() as i64
}

pub fn league(rating: i64) -> &'static str {
    match rating {
        r if r < 1100 => "Bronze",
        r if r < 1250 => "Silver",
        r if r < 1400 => "Gold",
        r if r < 1550 => "Platinum",
        _ => "Diamond",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beating_stronger_bot_gives_more() {
        assert!(rating_delta(1000, 1500, 1.0) > rating_delta(1000, 900, 1.0));
        assert!(rating_delta(1000, 900, 0.0) < 0);
    }
}
