// Проверка решений: запускает код игрока на тестах задачи и выносит вердикт.
mod check;
mod handlers;
mod model;
mod process;

pub use check::check_solution;
pub use handlers::routes;
pub use model::{RunRequest, RunResult, Verdict};
