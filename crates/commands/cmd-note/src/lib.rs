use anyhow::Result;
use jarvis_core::{Command, Context, Output};

/// Команда `note`: добавить, показать и удалить заметку.
///
/// Заметки хранятся в JSON-файле, не в SQLite (см. `docs/DECISIONS.md`).
#[derive(Debug, Default)]
pub struct NoteCommand;

impl NoteCommand {
    /// Создать команду.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Command for NoteCommand {
    fn name(&self) -> &'static str {
        "note"
    }

    fn description(&self) -> &'static str {
        "заметки: добавить, показать, удалить"
    }

    fn matches(&self, _input: &str) -> bool {
        false
    }

    async fn run(&self, _input: &str, _ctx: &Context) -> Result<Output> {
        Err(anyhow::anyhow!("not implemented"))
    }
}
