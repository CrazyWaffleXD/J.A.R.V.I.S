use std::path::PathBuf;

use crate::context::Context;
use crate::error::Result;

/// Результат выполнения команды.
#[derive(Debug, Clone)]
pub struct Output {
    /// Текст, который показывается пользователю.
    pub text: String,
    /// Запрос на озвучку результата, если он нужен.
    pub speech: Option<Speech>,
}

/// Что именно озвучить: готовый файл или байты.
#[derive(Debug, Clone)]
pub enum Speech {
    /// Аудиофайл на диске.
    File(PathBuf),
    /// Аудио в памяти.
    Bytes(Vec<u8>),
}

/// Команда, которую умеет выполнять jarvis.
///
/// Реализации живут в отдельных крейтах `commands/*` и регистрируются
/// в [`crate::Dispatcher`].
#[async_trait::async_trait]
pub trait Command: Send + Sync {
    /// Имя, под которым команда вызывается из REPL.
    fn name(&self) -> &'static str;

    /// Краткое описание для справки.
    fn description(&self) -> &'static str;

    /// Подходит ли команда под введённую строку.
    fn matches(&self, input: &str) -> bool;

    /// Выполнить команду с полной строкой ввода и контекстом сессии.
    async fn run(&self, input: &str, ctx: &Context) -> Result<Output>;
}
