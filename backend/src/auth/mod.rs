// Аккаунты: регистрация, вход, токены и проверка «кто делает запрос».
mod handlers;
mod password;
mod token;
mod user;

pub use handlers::routes;
pub use user::AuthUser;
