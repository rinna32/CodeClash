// Задачи: модель, стартовый набор, запросы к базе и HTTP-обработчики.
mod data;
mod handlers;
mod model;
mod repo;

pub use data::insert_seed_tasks_if_empty;
#[cfg(test)]
pub use data::seed_tasks;
pub use handlers::routes;
pub use model::{Difficulty, Mode, NewTask, Task, TaskDetail, TestCase};
pub use repo::{find_by_id, insert, pick_for_match};
