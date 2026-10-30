use anyhow::Result;
use jarvis_core::{Command, Context};

/// Команда `open`: открыть URL в браузере по умолчанию.
#[derive(Debug, Default)]
pub struct OpenCommand;

impl OpenCommand {
    /// Создать команду.
    pub fn new() -> Self {
        Self
    }
}

impl Command for OpenCommand {
    fn name(&self) -> &'static str {
        "open"
    }

    fn description(&self) -> &'static str {
        "открыть URL в браузере по умолчанию"
    }

    fn execute(&self, _ctx: &mut Context, _args: &[String]) -> Result<()> {
        Err(anyhow::anyhow!("not implemented"))
    }
}
