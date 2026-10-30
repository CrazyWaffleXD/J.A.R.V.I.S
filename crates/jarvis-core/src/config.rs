use serde::Deserialize;

/// Пользовательская конфигурация jarvis.
///
/// Поля появятся на следующих шагах; пока структура пустая, чтобы зафиксировать
/// способ загрузки (JSON через `serde`).
#[derive(Debug, Default, Deserialize)]
pub struct Config {}
