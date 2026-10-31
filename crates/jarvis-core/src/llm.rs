//! Абстракция LLM-провайдера.
//!
//! Ядро не знает про конкретные API: реализации живут в отдельных крейтах
//! (`jarvis-llm-mistral` и будущие), а команды получают провайдера из
//! контекста сессии. Без провайдера jarvis работает только командами.

use async_trait::async_trait;

use crate::Result;

/// Провайдер больших языковых моделей.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Получить ответ модели на `prompt`.
    ///
    /// `model` переопределяет модель провайдера; `None` — модель по умолчанию.
    async fn complete(&self, prompt: &str, model: Option<&str>) -> Result<String>;
}
