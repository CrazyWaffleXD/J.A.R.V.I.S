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

/// Системный промпт по умолчанию: роль Джарвиса и протокол `__CMD__:`.
///
/// Модель не объясняет, как выполнить действие, а просит CLI выполнить
/// команду: REPL распознаёт строку `__CMD__:<команда>` и передаёт её
/// диспетчеру (см. `jarvis-cli`).
pub const DEFAULT_SYSTEM_PROMPT: &str =
    "Ты — Джарвис, вежливый и немного саркастичный ИИ-ассистент. \
Ты работаешь в консольном приложении на Rust. \
Твои доступные команды: open <url> (открыть сайт), weather (погода), \
voice (голосовой режим), help.\n\n\
ВАЖНОЕ ПРАВИЛО: Если пользователь просит выполнить действие \
(открыть сайт, узнать погоду), ты НЕ должен объяснять, как это \
сделать. Вместо этого верни СТРОГО строку в формате:\n\
__CMD__:<команда>\n\
Например, если просят 'открой ютуб', верни: __CMD__:открой ютуб.\n\
Если просят 'какая погода', верни: __CMD__:weather.\n\
Если это обычный вопрос, просто ответь текстом.";

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
    /// Модель провайдера; `None` — модель по умолчанию у провайдера.
    #[serde(default)]
    pub model: Option<String>,
    /// Системный промпт; по умолчанию [`DEFAULT_SYSTEM_PROMPT`].
    /// Пустая строка отключает системный промпт.
    #[serde(default = "default_system_prompt")]
    pub system_prompt: Option<String>,
}

fn default_llm_provider() -> String {
    "mistral".to_string()
}

fn default_llm_api_key_env() -> String {
    "MISTRAL_API_KEY".to_string()
}

fn default_system_prompt() -> Option<String> {
    Some(DEFAULT_SYSTEM_PROMPT.to_string())
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_llm_provider(),
            api_key_env: default_llm_api_key_env(),
            model: None,
            system_prompt: default_system_prompt(),
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

#[cfg(test)]
mod tests {
    use super::{Config, DEFAULT_SYSTEM_PROMPT};

    #[test]
    fn system_prompt_defaults_in_toml() {
        let config: Config = toml::from_str("[llm]\nprovider = \"mistral\"\n").unwrap();
        let llm = config.llm.unwrap();
        assert_eq!(llm.system_prompt.as_deref(), Some(DEFAULT_SYSTEM_PROMPT));
    }

    #[test]
    fn system_prompt_can_be_overridden_or_disabled() {
        let custom: Config = toml::from_str("[llm]\nsystem_prompt = \"Будь краток.\"\n").unwrap();
        assert_eq!(
            custom.llm.unwrap().system_prompt.as_deref(),
            Some("Будь краток.")
        );

        // Пустая строка — явный отказ от системного промпта.
        let disabled: Config = toml::from_str("[llm]\nsystem_prompt = \"\"\n").unwrap();
        assert_eq!(disabled.llm.unwrap().system_prompt.as_deref(), Some(""));
    }
}
