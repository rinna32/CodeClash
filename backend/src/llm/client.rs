// Клиент OpenAI-совместимого API. Такой формат понимают Groq, OpenRouter, Alibaba Model Studio
// и локальная Ollama — провайдер меняется одной строкой в .env.
use std::time::Duration;

use serde_json::{Value, json};

use crate::config::LlmConfig;
use crate::error::AppError;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(180); // бесплатные модели бывают медленными
const TEMPERATURE: f64 = 0.7; // чуть творчества, чтобы задачи не повторялись
const MAX_RATE_LIMIT_RETRIES: u32 = 3; // сколько раз переждать лимит 429 и повторить
const DEFAULT_RETRY_WAIT: Duration = Duration::from_secs(20); // если провайдер не сказал, сколько ждать
const MAX_RETRY_WAIT: Duration = Duration::from_secs(60);

pub struct LlmClient {
    http: reqwest::Client,
    config: LlmConfig,
}

impl LlmClient {
    pub fn new(config: LlmConfig) -> LlmClient {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("Не удалось создать HTTP-клиент");

        LlmClient { http, config }
    }

    pub fn model(&self) -> &str {
        &self.config.model
    }

    // Просим ответ строго в JSON и сразу его разбираем
    pub async fn chat_json(&self, system: &str, user: &str) -> Result<Value, AppError> {
        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );
        let body = json!({
            "model": self.config.model,
            "temperature": TEMPERATURE,
            "response_format": { "type": "json_object" },
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user },
            ],
        });

        // У бесплатных тарифов лимит токенов в минуту (у Groq — 1000 на ответ).
        // Задача с решением бота в него не всегда влезает — пережидаем и повторяем
        let mut retries = 0;
        let response = loop {
            let mut request = self.http.post(&url).json(&body);
            if !self.config.api_key.is_empty() {
                request = request.bearer_auth(&self.config.api_key);
            }

            let response = request
                .send()
                .await
                .map_err(|err| AppError::Llm(format!("не удалось связаться с {url}: {err}")))?;

            if response.status() != reqwest::StatusCode::TOO_MANY_REQUESTS
                || retries == MAX_RATE_LIMIT_RETRIES
            {
                break response;
            }

            let wait = retry_wait(&response);
            retries += 1;
            log::info!(
                "Лимит нейросети, ждём {} с (попытка {retries}/{MAX_RATE_LIMIT_RETRIES})",
                wait.as_secs()
            );
            tokio::time::sleep(wait).await;
        };

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|err| AppError::Llm(err.to_string()))?;

        if !status.is_success() {
            let short: String = text.chars().take(300).collect();
            return Err(AppError::Llm(format!("{status}: {short}")));
        }

        let content = serde_json::from_str::<Value>(&text).ok().and_then(|value| {
            value["choices"][0]["message"]["content"]
                .as_str()
                .map(str::to_string)
        });

        match content {
            Some(content) => parse_json_content(&content),
            None => Err(AppError::Llm(
                "в ответе нет choices[0].message.content".to_string(),
            )),
        }
    }
}

// Сколько ждать после 429: провайдер обычно пишет это в заголовке retry-after (в секундах)
fn retry_wait(response: &reqwest::Response) -> Duration {
    let seconds = response
        .headers()
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<f64>().ok());

    match seconds {
        // +1 с запаса: лимит считается скользящим окном
        Some(seconds) => Duration::from_secs_f64(seconds + 1.0).min(MAX_RETRY_WAIT),
        None => DEFAULT_RETRY_WAIT,
    }
}

// Модели не всегда слушаются «только JSON»: бывают ```json-обёртки и текст вокруг.
// Берём всё от первой { до последней }.
pub fn parse_json_content(content: &str) -> Result<Value, AppError> {
    let start = content.find('{');
    let end = content.rfind('}');

    let json_text = match (start, end) {
        (Some(start), Some(end)) if start < end => &content[start..=end],
        _ => return Err(AppError::Llm("в ответе нет JSON-объекта".to_string())),
    };

    serde_json::from_str(json_text)
        .map_err(|err| AppError::Llm(format!("ответ — не валидный JSON: {err}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_inside_markdown_fence() {
        let value = parse_json_content("Вот задача:\n```json\n{\"title\": \"A\"}\n```").unwrap();
        assert_eq!(value["title"], "A");
    }

    #[test]
    fn rejects_text_without_json() {
        assert!(parse_json_content("не могу помочь").is_err());
    }
}
