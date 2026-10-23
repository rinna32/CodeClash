// Настройки сервера из переменных окружения (.env).
use std::env;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8000; // 8080 часто занят локальным nginx
const DEFAULT_FRONTEND_ORIGIN: &str = "http://localhost:3000"; // dev-сервер Nuxt
const DEFAULT_DATABASE_URL: &str = "sqlite://codeclash.db?mode=rwc"; // rwc — создать файл, если его нет
const DEV_JWT_SECRET: &str = "dev-secret-change-me";
const DEFAULT_LLM_BASE_URL: &str = "https://api.groq.com/openai/v1";
const DEFAULT_LLM_MODEL: &str = "qwen/qwen3.8-27b";

pub struct Config {
    pub host: String,
    pub port: u16,
    pub frontend_origin: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub llm: Option<LlmConfig>, // None — ключа нет, генерация задач выключена
}

// Любой OpenAI-совместимый API: OpenRouter, Alibaba Model Studio, локальная Ollama
#[derive(Clone)]
pub struct LlmConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl Config {
    pub fn from_env() -> Config {
        let jwt_secret = env::var("JWT_SECRET").unwrap_or(DEV_JWT_SECRET.to_string());
        if jwt_secret == DEV_JWT_SECRET {
            log::warn!("JWT_SECRET не задан — используется тестовый секрет, для деплоя так нельзя");
        }

        Config {
            host: env::var("HOST").unwrap_or(DEFAULT_HOST.to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|port| port.parse().ok())
                .unwrap_or(DEFAULT_PORT),
            frontend_origin: env::var("FRONTEND_ORIGIN")
                .unwrap_or(DEFAULT_FRONTEND_ORIGIN.to_string()),
            database_url: env::var("DATABASE_URL").unwrap_or(DEFAULT_DATABASE_URL.to_string()),
            jwt_secret,
            llm: llm_from_env(),
        }
    }
}

// Ключ обязателен для облачных API. Для Ollama ключ не нужен — достаточно задать LLM_BASE_URL.
fn llm_from_env() -> Option<LlmConfig> {
    let api_key = env::var("LLM_API_KEY").unwrap_or_default();
    let base_url = env::var("LLM_BASE_URL").unwrap_or(DEFAULT_LLM_BASE_URL.to_string());

    if api_key.is_empty() && base_url == DEFAULT_LLM_BASE_URL {
        return None;
    }

    Some(LlmConfig {
        base_url,
        api_key,
        model: env::var("LLM_MODEL").unwrap_or(DEFAULT_LLM_MODEL.to_string()),
    })
}
