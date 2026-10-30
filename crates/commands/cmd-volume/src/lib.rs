use anyhow::Result;
use jarvis_core::{Command, Context};

/// Команда `volume`: изменить системную громкость.
#[derive(Debug, Default)]
pub struct VolumeCommand;

impl VolumeCommand {
    /// Создать команду.
    pub fn new() -> Self {
        Self
    }
}

impl Command for VolumeCommand {
    fn name(&self) -> &'static str {
        "volume"
    }

    fn description(&self) -> &'static str {
        "изменить системную громкость"
    }

    fn execute(&self, _ctx: &mut Context, _args: &[String]) -> Result<()> {
        Err(anyhow::anyhow!("not implemented"))
    }
}
