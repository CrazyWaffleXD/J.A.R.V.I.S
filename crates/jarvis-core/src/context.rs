use crate::config::Config;

/// Состояние одной сессии jarvis: то, что команды читают и меняют во время работы.
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// Пользовательская конфигурация.
    pub config: Config,
}
