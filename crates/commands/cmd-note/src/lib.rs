use anyhow::Result;
use jarvis_core::{Command, Context};

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

impl Command for NoteCommand {
    fn name(&self) -> &'static str {
        "note"
    }

    fn description(&self) -> &'static str {
        "заметки: добавить, показать, удалить"
    }

    fn execute(&self, _ctx: &mut Context, _args: &[String]) -> Result<()> {
        Err(anyhow::anyhow!("not implemented"))
    }
}
