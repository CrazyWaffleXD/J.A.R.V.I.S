use std::path::Path;

use serde::Deserialize;

use crate::error::Result;
use crate::stt::SttConfig;

/// Настройки погоды.
#[derive(Clone, Debug, Deserialize)]
pub struct WeatherConfig {
    /// Город по умолчанию, если он не указан в команде.
    pub default_city: String,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            default_city: "Moscow".to_string(),
        }
    }
}

/// Пользовательская конфигурация jarvis.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    /// Секция `[weather]`.
    #[serde(default)]
    pub weather: WeatherConfig,
    /// Секция `[stt]`; `None` — голосовой ввод не настроен.
    #[serde(default)]
    pub stt: Option<SttConfig>,
    /// Секция `[llm]`; `None` — LLM не настроена, работают только команды.
    #[serde(default)]
    pub llm: Option<LlmConfig>,
}

/// Настройки LLM из секции `[llm]`.
///
/// Ключ API в конфиге не хранится: здесь только имя переменной окружения.
#[derive(Clone, Debug, Deserialize)]
pub struct LlmConfig {
    /// Имя провайдера, пока поддерживается только `mistral`.
    #[serde(default = "default_llm_provider")]
    pub provider: String,
    /// Переменная окружения с ключом API.
    #[serde(default = "default_llm_api_key_env")]
    pub api_key_env: String,
}

fn default_llm_provider() -> String {
    "mistral".to_string()
}

fn default_llm_api_key_env() -> String {
    "MISTRAL_API_KEY".to_string()
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_llm_provider(),
            api_key_env: default_llm_api_key_env(),
        }
    }
}

impl Config {
    /// Загрузить конфигурацию из TOML-файла.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&text)?)
    }
}
