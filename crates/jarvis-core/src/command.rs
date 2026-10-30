use crate::context::Context;
use crate::error::Result;

/// Команда, которую умеет выполнять jarvis.
///
/// Реализации живут в отдельных крейтах `commands/*` и регистрируются
/// в [`crate::Dispatcher`].
pub trait Command: Send + Sync {
    /// Имя, под которым команда вызывается из REPL.
    fn name(&self) -> &'static str;

    /// Краткое описание для справки.
    fn description(&self) -> &'static str;

    /// Выполнить команду с аргументами, разобранными из строки ввода.
    fn execute(&self, ctx: &mut Context, args: &[String]) -> Result<()>;
}
