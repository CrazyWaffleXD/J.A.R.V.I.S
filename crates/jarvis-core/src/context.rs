use std::sync::Arc;

use crate::config::Config;
use crate::llm::LlmProvider;
use crate::stt::SpeechToText;

/// Состояние одной сессии jarvis: то, что команды читают и меняют во время работы.
#[derive(Clone, Default)]
pub struct Context {
    /// Пользовательская конфигурация.
    pub config: Config,
    /// Готовый к работе STT-провайдер; `None` — голосовой ввод недоступен.
    pub stt: Option<Arc<dyn SpeechToText>>,
    /// Готовый LLM-провайдер; `None` — умные ответы недоступны.
    pub llm: Option<Arc<dyn LlmProvider>>,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("config", &self.config)
            .field("stt", &self.stt.as_ref().map(|_| "<SpeechToText>"))
            .field("llm", &self.llm.as_ref().map(|_| "<LlmProvider>"))
            .finish()
    }
}
