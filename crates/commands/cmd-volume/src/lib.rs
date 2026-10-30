use anyhow::Result;
use jarvis_core::{Command, Context, Output};

/// Команда `volume`: изменить системную громкость.
#[derive(Debug, Default)]
pub struct VolumeCommand;

impl VolumeCommand {
    /// Создать команду.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Command for VolumeCommand {
    fn name(&self) -> &'static str {
        "volume"
    }

    fn description(&self) -> &'static str {
        "изменить системную громкость"
    }

    fn matches(&self, _input: &str) -> bool {
        false
    }

    async fn run(&self, _input: &str, _ctx: &Context) -> Result<Output> {
        Err(anyhow::anyhow!("not implemented"))
    }
}
