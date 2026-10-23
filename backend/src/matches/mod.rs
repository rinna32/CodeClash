// Матч против бота: таймер, расписание бота, правила победы, рейтинг.
mod bot;
mod handlers;
mod model;
mod outcome;
pub mod rating;
mod repo;

pub use handlers::{routes, settle_abandoned};
