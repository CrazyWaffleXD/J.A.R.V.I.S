use anyhow::Result;
use jarvis_core::{Command, Context, Output};

/// Команда `weather`: текущая погода для заданного города.
///
/// Источник данных (open-meteo или wttr.in) появится на следующем шаге.
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
        "показать текущую погоду"
    }

    fn matches(&self, _input: &str) -> bool {
        false
    }

    async fn run(&self, _input: &str, _ctx: &Context) -> Result<Output> {
        Err(anyhow::anyhow!("not implemented"))
    }
}
