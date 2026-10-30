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
}

impl Config {
    /// Загрузить конфигурацию из TOML-файла.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&text)?)
    }
}
