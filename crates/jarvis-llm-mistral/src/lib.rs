//! LLM-провайдер Mistral через HTTP API.
//!
//! Ключ API берётся только из переменной окружения (по умолчанию
//! `MISTRAL_API_KEY`) — в `config.toml` и в репозитории его нет.
//!
//! Без ключа провайдер не создаётся: `jarvis` продолжает работать одними
//! командами (см. `build_mistral`).

use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context as _, Result};
use async_trait::async_trait;
use jarvis_core::llm::LlmProvider;
use jarvis_core::LlmConfig;

/// Модель по умолчанию: быстрая и дешёвая, для «как дела?» хватает.
const DEFAULT_MODEL: &str = "mistral-small-latest";

/// Базовый URL API Mistral.
const DEFAULT_BASE_URL: &str = "https://api.mistral.ai/v1";

/// Переменная окружения с ключом, если в конфиге имя не задано.
const DEFAULT_API_KEY_ENV: &str = "MISTRAL_API_KEY";

/// Таймаут запроса: LLM не должна подвешивать REPL.
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

/// Сколько символов тела ошибки показывать (там бывает JSON с причиной).
const MAX_ERROR_BODY: usize = 300;

/// Клиент Mistral: `POST /chat/completions`, один пользовательский вопрос.
pub struct MistralClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl MistralClient {
    /// Создать клиент с готовым ключом и URL по умолчанию.
    pub fn new(api_key: String) -> Self {
        Self {
            client: build_client(),
            api_key,
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Создать клиент по переменной окружения `MISTRAL_API_KEY`.
    pub fn from_env() -> Result<Self> {
        let key = std::env::var(DEFAULT_API_KEY_ENV)
            .map_err(|_| anyhow!("{DEFAULT_API_KEY_ENV} не установлен"))?;
        if key.trim().is_empty() {
            return Err(anyhow!("{DEFAULT_API_KEY_ENV} пуст"));
        }
        Ok(Self::new(key))
    }

    /// Переопределить базовый URL (самопроверка, будущие прокси).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

#[async_trait]
impl LlmProvider for MistralClient {
    async fn complete(&self, prompt: &str, model: Option<&str>) -> anyhow::Result<String> {
        let model = model.unwrap_or(DEFAULT_MODEL);
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let body = ChatRequest {
            model,
            messages: vec![ChatMessage {
                role: "user",
                content: prompt,
            }],
        };

        tracing::debug!(model, "запрос к Mistral");

        // `.context`, а не `map_err` со строкой: anyhow сохраняет цепочку
        // источников (TLS, DNS, таймаут, hyper), и `{e:#}` в REPL показывает
        // её целиком, а не только «error sending request».
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .context("не удалось отправить запрос в Mistral")?;

        let status = response.status();
        let raw = response
            .text()
            .await
            .context("не удалось прочитать ответ Mistral")?;

        if !status.is_success() {
            return Err(anyhow!(
                "Mistral API вернул {status}: {}",
                truncate(&raw, MAX_ERROR_BODY)
            ));
        }

        let parsed: ChatResponse =
            serde_json::from_str(&raw).context("не удалось разобрать ответ Mistral")?;

        let text = parsed
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .unwrap_or_default();

        if text.trim().is_empty() {
            return Err(anyhow!("Mistral вернул пустой ответ"));
        }

        Ok(text)
    }
}

/// Построить провайдера Mistral по секции `[llm]`.
///
/// Ключ читается из `cfg.api_key_env` (по умолчанию `MISTRAL_API_KEY`).
/// Если переменной нет — ошибка, и CLI работает без LLM.
pub fn build_mistral(cfg: &LlmConfig) -> Result<Arc<dyn LlmProvider>> {
    let env_name = cfg.api_key_env.trim();
    let env_name = if env_name.is_empty() {
        DEFAULT_API_KEY_ENV
    } else {
        env_name
    };

    let api_key = std::env::var(env_name).map_err(|_| {
        anyhow!("переменная окружения {env_name} не установлена (см. README, раздел «LLM»)")
    })?;

    if api_key.trim().is_empty() {
        return Err(anyhow!(
            "переменная окружения {env_name} пуста (см. README, раздел «LLM»)"
        ));
    }

    tracing::debug!(provider = %cfg.provider, env = env_name, "Mistral настроен");
    Ok(Arc::new(MistralClient::new(api_key)))
}

/// HTTP-клиент с таймаутом.
fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .unwrap_or_default()
}

/// Обрезать длинное тело ошибки, не разрывая UTF-8.
fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }

    let mut out: String = text.chars().take(limit).collect();
    out.push('…');
    out
}

#[derive(serde::Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
}

#[derive(serde::Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(serde::Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(serde::Deserialize)]
struct ChatChoice {
    message: ChatMessageOut,
}

#[derive(serde::Deserialize)]
struct ChatMessageOut {
    #[serde(default)]
    content: String,
}

#[cfg(test)]
mod tests {
    use super::truncate;

    #[test]
    fn truncates_by_chars() {
        assert_eq!(truncate("привет", 10), "привет");
        assert_eq!(truncate("привет", 3), "при…");
    }
}
