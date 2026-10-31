//! Wake word «Джарвис» на Vosk.
//!
//! Детектор слушает микрофон и реагирует только на своё имя: распознавание
//! ограничено грамматикой из [`GRAMMAR`], а всё остальное У Вosk уходит в
//! `[unk]`. Это позволяет постоянно слушать микрофон и не выполнять случайные
//! фразы: сначала wake word, потом отдельная запись команды.
//!
//! Нативная библиотека `libvosk` не входит в репозиторий — её ставит
//! пользователь (см. README).

use std::sync::Arc;

use anyhow::anyhow;
use vosk::{Model, Recognizer};

/// Слова, которые считаются обращением к Джарвису.
const WAKE_PHRASES: &[&str] = &["джарвис", "джарвиз", "jarvis"];

/// Грамматика распознавателя: только wake-слова и `[unk]` для остального.
///
/// `[unk]` обязателен: без него Vosk попытается распознать любую речь как
/// одно из wake-слов и будет срабатывать на посторонние фразы.
const GRAMMAR: &[&str] = &["джарвис", "джарвиз", "jarvis", "[unk]"];

/// Детектор wake word: одна загруженная модель Vosk на всё время работы.
pub struct WakeWordDetector {
    model: Arc<Model>,
    sample_rate: u32,
    grammar: Vec<String>,
}

impl WakeWordDetector {
    /// Загрузить модель и подготовить грамматику.
    ///
    /// `sample_rate` должен совпадать с частотой, на которой обучена модель
    /// (для `vosk-model-small-ru-0.22` — 16000).
    pub fn new(model_path: &std::path::Path, sample_rate: u32) -> anyhow::Result<Self> {
        if sample_rate == 0 {
            return Err(anyhow!("sample_rate должен быть больше нуля"));
        }

        let path = model_path
            .to_str()
            .ok_or_else(|| anyhow!("non-utf8 model path: {}", model_path.display()))?;

        tracing::debug!(path = %model_path.display(), "загружаю модель Vosk для wake word");
        let model = Model::new(path).ok_or_else(|| {
            anyhow!(
                "не удалось загрузить модель Vosk из {}",
                model_path.display()
            )
        })?;

        Ok(Self {
            model: Arc::new(model),
            sample_rate,
            grammar: GRAMMAR.iter().map(|word| (*word).to_string()).collect(),
        })
    }

    /// Блокирующий: слушает микрофон и возвращает `Ok(())`, когда услышал
    /// любое из wake-слов.
    ///
    /// Внутри `spawn_blocking`: запись с микрофона и распознавание
    /// синхронные.
    pub async fn wait_for_wake(&self) -> anyhow::Result<()> {
        let model = Arc::clone(&self.model);
        let sample_rate = self.sample_rate;
        let grammar = self.grammar.clone();

        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let mut recognizer = Recognizer::new_with_grammar(&model, sample_rate as f32, &grammar)
                .ok_or_else(|| anyhow!("не удалось создать распознаватель Vosk с грамматикой"))?;

            let mut detected = false;

            jarvis_audio::record_stream(sample_rate, |chunk| {
                if recognizer.accept_waveform(chunk).is_err() {
                    return false;
                }

                // Частичный результат приходит ещё во время произнесения —
                // по нему срабатываем быстрее, чем ждать конца фразы.
                if is_wake(recognizer.partial_result().partial) {
                    detected = true;
                    return false;
                }

                // `result` (не `final_result`) отдаёт последнюю завершённую
                // фразу и не закрывает распознаватель.
                if let Some(result) = recognizer.result().single() {
                    if is_wake(result.text) {
                        detected = true;
                        return false;
                    }
                }

                true
            })?;

            if detected {
                Ok(())
            } else {
                Err(anyhow!("поток завершился без wake word"))
            }
        })
        .await?
    }
}

/// Есть ли в распознанном тексте обращение к Джарвису.
///
/// Сравниваем по словам, а не по подстроке: «джарвисом» не считается wake
/// word, а «джарвис, джарвис» — считается. Грамматика ограничивает выдачу
/// распознавателя, поэтому лишние слова в тексте маловероятны; если имя всё
/// же прозвучало, лучше среагировать.
pub fn is_wake(text: &str) -> bool {
    let text = text.trim().to_lowercase();
    if text.is_empty() {
        return false;
    }

    text.split(|c: char| !c.is_alphabetic())
        .any(|word| WAKE_PHRASES.contains(&word))
}

#[cfg(test)]
mod tests {
    use super::is_wake;

    #[test]
    fn detects_exact_wake_words() {
        assert!(is_wake("джарвис"));
        assert!(is_wake("Джарвис"));
        assert!(is_wake("джарвиз"));
        assert!(is_wake("jarvis"));
        assert!(is_wake("джарвис,"));
        assert!(is_wake("джарвис джарвис"));
    }

    #[test]
    fn ignores_other_speech() {
        assert!(!is_wake(""));
        assert!(!is_wake("погода"));
        assert!(!is_wake("джарвисом"));
    }
}
