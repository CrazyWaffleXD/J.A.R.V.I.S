use anyhow::Result;
use jarvis_core::{Command, Context};

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

impl Command for WeatherCommand {
    fn name(&self) -> &'static str {
        "weather"
    }

    fn description(&self) -> &'static str {
        "показать текущую погоду"
    }

    fn execute(&self, _ctx: &mut Context, _args: &[String]) -> Result<()> {
        Err(anyhow::anyhow!("not implemented"))
    }
}
