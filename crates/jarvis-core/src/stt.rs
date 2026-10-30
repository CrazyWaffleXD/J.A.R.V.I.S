//! Абстракция распознавания речи (STT).
//!
//! Трейт и настройки живут в ядре, реализации — в отдельных крейтах
//! (`jarvis-stt-vosk` и будущие). Ядро не знает про Vosk и не зависит
//! от крейтов-реализаций.

use std::path::PathBuf;

use async_trait::async_trait;

use crate::Result;

/// Распознавание речи из моно-PCM.
#[async_trait]
pub trait SpeechToText: Send + Sync {
    /// Распознать текст из моно-сэмплов `f32` в диапазоне `[-1.0, 1.0]`.
    ///
    /// `sample_rate` — частота дискретизации входного аудио.
    async fn transcribe(&self, audio: &[f32], sample_rate: u32) -> Result<String>;
}

/// Настройки STT из секции `[stt]` конфига.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct SttConfig {
    /// Путь к каталогу с моделью Vosk.
    pub model_path: PathBuf,
    /// Частота дискретизации, ожидаемая моделью.
    pub sample_rate: u32,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::new(),
            sample_rate: 16000,
        }
    }
}
