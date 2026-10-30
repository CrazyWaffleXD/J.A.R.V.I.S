use std::time::Duration;

use anyhow::{anyhow, Context as _, Result};
use jarvis_core::{persona, Command, Context, Output};
use serde_json::Value;

/// Таймаут на один HTTP-запрос: wttr.in не должен подвешивать REPL.
const HTTP_TIMEOUT: Duration = Duration::from_secs(5);

/// Команда `weather`: текущая погода по open-meteo, с fallback на wttr.in.
#[derive(Debug, Default)]
pub struct WeatherCommand;

impl WeatherCommand {
    /// Создать команду.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Command for WeatherCommand {
    fn name(&self) -> &'static str {
        "weather"
    }

    fn description(&self) -> &'static str {
        "погода (город из config, или weather <город>)"
    }

    fn matches(&self, input: &str) -> bool {
        input
            .split_whitespace()
            .next()
            .map(|word| {
                let word = word.to_lowercase();
                word == "weather" || word == "погода"
            })
            .unwrap_or(false)
    }

    async fn run(&self, input: &str, ctx: &Context) -> Result<Output> {
        let rest = input
            .split_once(char::is_whitespace)
            .map(|(_, r)| r.trim())
            .filter(|s| !s.is_empty());
        let city = rest.unwrap_or(&ctx.config.weather.default_city);

        match try_open_meteo(city).await {
            Ok(output) => Ok(output),
            Err(e1) => {
                tracing::warn!(?e1, "open-meteo failed, trying wttr.in");
                match try_wttr_in(city).await {
                    Ok(output) => Ok(output),
                    Err(e2) => Err(anyhow!("open-meteo: {e1:#}; wttr.in: {e2:#}")),
                }
            }
        }
    }
}

/// HTTP-клиент с общим таймаутом.
fn http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .map_err(|e| anyhow!("не смог создать HTTP-клиент: {e}"))
}

/// Основной провайдер: geocoding + forecast open-meteo.
async fn try_open_meteo(city: &str) -> Result<Output> {
    let client = http_client()?;
    let (lat, lon, city_name) = geocode(&client, city).await?;
    let (temp, code) = forecast(&client, lat, lon).await?;

    Ok(Output {
        text: persona::success(&format!(
            "{city_name}: {temp:.0}°C, {}",
            weather_description(code)
        )),
        speech: None,
    })
}

/// Fallback: wttr.in принимает имя города напрямую, без geocoding.
async fn try_wttr_in(city: &str) -> Result<Output> {
    let client = http_client()?;
    let url = format!("https://wttr.in/{}?format=j1", urlencoding::encode(city));

    let body: Value = client
        .get(&url)
        .send()
        .await
        .context("wttr.in")?
        .error_for_status()
        .map_err(|e| anyhow!("wttr.in вернул ошибку: {e}"))?
        .json()
        .await
        .map_err(|e| anyhow!("не разобрал ответ wttr.in: {e}"))?;

    let current = body
        .get("current_condition")
        .and_then(Value::as_array)
        .and_then(|conditions| conditions.first())
        .ok_or_else(|| anyhow!("в ответе wttr.in нет current_condition"))?;

    let temp = current
        .get("temp_C")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("в ответе wttr.in нет temp_C"))?;
    let desc = current
        .get("weatherDesc")
        .and_then(Value::as_array)
        .and_then(|descriptions| descriptions.first())
        .and_then(|item| item.get("value"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("в ответе wttr.in нет weatherDesc"))?
        .trim();

    Ok(Output {
        text: persona::success(&format!("{city}: {temp}°C, {desc}")),
        speech: None,
    })
}

/// Найти координаты и каноническое имя города.
async fn geocode(client: &reqwest::Client, city: &str) -> Result<(f64, f64, String)> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=ru&format=json",
        urlencoding::encode(city)
    );

    let response = client.get(&url).send().await.context("geocoding")?;
    let body: Value = response
        .error_for_status()
        .map_err(|e| anyhow!("геокодинг вернул ошибку для «{city}»: {e}"))?
        .json()
        .await
        .map_err(|e| anyhow!("не разобрал ответ геокодинга для «{city}»: {e}"))?;

    let first = body
        .get("results")
        .and_then(Value::as_array)
        .and_then(|results| results.first())
        .ok_or_else(|| anyhow!("город не найден: {city}"))?;

    let lat = first
        .get("latitude")
        .and_then(Value::as_f64)
        .ok_or_else(|| anyhow!("в ответе геокодинга нет latitude для «{city}»"))?;
    let lon = first
        .get("longitude")
        .and_then(Value::as_f64)
        .ok_or_else(|| anyhow!("в ответе геокодинга нет longitude для «{city}»"))?;
    let name = first
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(city)
        .to_string();

    Ok((lat, lon, name))
}

/// Текущая температура и WMO weather_code.
async fn forecast(client: &reqwest::Client, lat: f64, lon: f64) -> Result<(f64, u8)> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}\
         &current=temperature_2m,weather_code&timezone=auto"
    );

    let response = client.get(&url).send().await.context("forecast")?;
    let body: Value = response
        .error_for_status()
        .map_err(|e| anyhow!("прогноз вернул ошибку: {e}"))?
        .json()
        .await
        .map_err(|e| anyhow!("не разобрал ответ прогноза: {e}"))?;

    let current = body
        .get("current")
        .ok_or_else(|| anyhow!("в ответе прогноза нет секции current"))?;

    let temp = current
        .get("temperature_2m")
        .and_then(Value::as_f64)
        .ok_or_else(|| anyhow!("в ответе прогноза нет temperature_2m"))?;
    let code = current
        .get("weather_code")
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("в ответе прогноза нет weather_code"))?;

    Ok((temp, code as u8))
}

/// Код WMO → описание на русском.
fn weather_description(code: u8) -> &'static str {
    match code {
        0 => "ясно",
        1..=3 => "переменная облачность",
        45 | 48 => "туман",
        51..=57 => "морось",
        61..=67 => "дождь",
        71..=77 => "снег",
        80..=82 => "ливень",
        85 | 86 => "снегопад",
        95..=99 => "гроза",
        _ => "неизвестно",
    }
}
