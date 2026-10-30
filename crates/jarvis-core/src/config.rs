use serde::Deserialize;

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
///
/// Загрузка из TOML появится позже; пока используется `Config::default()`.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    /// Секция `[weather]`.
    #[serde(default)]
    pub weather: WeatherConfig,
}
