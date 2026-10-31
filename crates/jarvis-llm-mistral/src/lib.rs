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
    /// Модель по умолчанию для этого клиента; `None` — [`DEFAULT_MODEL`].
    pub model: Option<String>,
    /// Системный промпт (роль и правила); `None` — только вопрос пользователя.
    pub system_prompt: Option<String>,
}

impl MistralClient {
    /// Создать клиент с готовым ключом и URL по умолчанию.
    pub fn new(api_key: String) -> Self {
        Self {
            client: build_client(),
            api_key,
            base_url: DEFAULT_BASE_URL.to_string(),
            model: None,
            system_prompt: None,
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

    /// Модель для запроса: аргумент → модель клиента → [`DEFAULT_MODEL`].
    fn resolve_model(&self, model: Option<&str>) -> String {
        model
            .filter(|m| !m.trim().is_empty())
            .map(str::to_string)
            .or_else(|| {
                self.model
                    .as_ref()
                    .map(|m| m.trim())
                    .filter(|m| !m.is_empty())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| DEFAULT_MODEL.to_string())
    }
}

#[async_trait]
impl LlmProvider for MistralClient {
    async fn complete(&self, prompt: &str, model: Option<&str>) -> anyhow::Result<String> {
        let model = self.resolve_model(model);
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let body = ChatRequest {
            model: model.as_str(),
            messages: chat_messages(self.system_prompt.as_deref(), prompt),
        };

        tracing::debug!(
            model = %model,
            system_prompt = self.system_prompt.is_some(),
            "запрос к Mistral"
        );

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
/// Ключ читается из `cfg.api_key_env` (по умолчанию `MISTRAL_API_KEY`),
/// модель — из `cfg.model` (пустая строка = модель по умолчанию).
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

    let model = cfg
        .model
        .as_ref()
        .map(|m| m.trim())
        .filter(|m| !m.is_empty())
        .map(str::to_string);

    // Пустая строка в конфиге — осознанный отказ от системного промпта.
    let system_prompt = cfg
        .system_prompt
        .as_ref()
        .filter(|sp| !sp.trim().is_empty())
        .cloned();

    tracing::debug!(
        provider = %cfg.provider,
        env = env_name,
        model = model.as_deref().unwrap_or(DEFAULT_MODEL),
        system_prompt = system_prompt.is_some(),
        "Mistral настроен"
    );

    let mut client = MistralClient::new(api_key);
    client.model = model;
    client.system_prompt = system_prompt;

    Ok(Arc::new(client))
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

/// Сообщения запроса: системный промпт (если задан) и вопрос пользователя.
///
/// Пустой промпт отбрасывается, чтобы не отправлять `content: " "`.
fn chat_messages<'a>(system_prompt: Option<&'a str>, prompt: &'a str) -> Vec<ChatMessage<'a>> {
    let mut messages = Vec::with_capacity(2);

    if let Some(system_prompt) = system_prompt.map(str::trim).filter(|sp| !sp.is_empty()) {
        messages.push(ChatMessage {
            role: "system",
            content: system_prompt,
        });
    }

    messages.push(ChatMessage {
        role: "user",
        content: prompt,
    });

    messages
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
    use super::{chat_messages, truncate, MistralClient, DEFAULT_MODEL};

    #[test]
    fn truncates_by_chars() {
        assert_eq!(truncate("привет", 10), "привет");
        assert_eq!(truncate("привет", 3), "при…");
    }

    #[test]
    fn system_prompt_goes_first() {
        let messages = chat_messages(Some("Ты — Джарвис."), "привет");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, "Ты — Джарвис.");
        assert_eq!(messages[1].role, "user");
        assert_eq!(messages[1].content, "привет");
    }

    #[test]
    fn empty_or_missing_system_prompt_is_skipped() {
        for system_prompt in [None, Some(""), Some("   ")] {
            let messages = chat_messages(system_prompt, "привет");
            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0].role, "user");
        }
    }

    #[test]
    fn model_priority() {
        let mut client = MistralClient::new("test".to_string());

        // Ничего не задано — дефолт провайдера.
        assert_eq!(client.resolve_model(None), DEFAULT_MODEL);

        // Модель клиента перекрывает дефолт.
        client.model = Some("mistral-large-latest".to_string());
        assert_eq!(client.resolve_model(None), "mistral-large-latest");

        // Аргумент вызова перекрывает модель клиента.
        assert_eq!(client.resolve_model(Some("custom")), "custom");

        // Пустая строка не считается выбором модели.
        assert_eq!(client.resolve_model(Some("   ")), "mistral-large-latest");

        client.model = Some("  ".to_string());
        assert_eq!(client.resolve_model(None), DEFAULT_MODEL);
    }
}
