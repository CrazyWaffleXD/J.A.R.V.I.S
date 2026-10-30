//! STT-провайдер на Vosk: офлайн-распознавание речи.
//!
//! Нативная библиотека `libvosk` не входит в репозиторий и не скачивается
//! автоматически — пользователь ставит её сам (см. README).

use std::sync::Arc;

use anyhow::anyhow;
use jarvis_core::stt::{SpeechToText, SttConfig};
use vosk::{Model, Recognizer};

/// Обёртка над моделью Vosk.
pub struct VoskStt {
    model: Arc<Model>,
}

impl VoskStt {
    /// Загрузить модель из каталога.
    pub fn new(model_path: &std::path::Path) -> anyhow::Result<Self> {
        let path = model_path
            .to_str()
            .ok_or_else(|| anyhow!("non-utf8 model path: {}", model_path.display()))?;

        tracing::debug!(path = %model_path.display(), "загружаю модель Vosk");
        let model = Model::new(path).ok_or_else(|| {
            anyhow!(
                "не удалось загрузить модель Vosk из {}",
                model_path.display()
            )
        })?;

        Ok(Self {
            model: Arc::new(model),
        })
    }
}

#[async_trait::async_trait]
impl SpeechToText for VoskStt {
    async fn transcribe(&self, audio: &[f32], sample_rate: u32) -> anyhow::Result<String> {
        let model = Arc::clone(&self.model);
        let audio: Vec<i16> = audio
            .iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .collect();

        // `Recognizer` создаётся внутри `spawn_blocking`, чтобы не тащить
        // его между потоками и не блокировать async-рантайм.
        tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
            let mut recognizer = Recognizer::new(&model, sample_rate as f32)
                .ok_or_else(|| anyhow!("не удалось создать распознаватель Vosk"))?;

            recognizer.accept_waveform(&audio)?;

            recognizer
                .final_result()
                .single()
                .map(|result| result.text.to_string())
                .ok_or_else(|| anyhow!("неожиданный формат результата Vosk"))
        })
        .await?
    }
}

/// Построить Vosk-провайдер по настройкам из конфига.
pub fn build_vosk(cfg: &SttConfig) -> anyhow::Result<Arc<dyn SpeechToText>> {
    if cfg.model_path.as_os_str().is_empty() {
        return Err(anyhow!("в config.toml не задан stt.model_path"));
    }

    Ok(Arc::new(VoskStt::new(&cfg.model_path)?))
}
