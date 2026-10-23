// Нейросеть (Qwen): пишет задачи, решает их за бота и разбирает решения игроков.
mod client;
mod generator;
mod handlers;
mod prompts;
mod review;

pub use client::LlmClient;
pub use handlers::routes;
pub use review::{Review, review_solution};
