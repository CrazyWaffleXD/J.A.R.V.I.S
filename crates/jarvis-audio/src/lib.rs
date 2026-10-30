//! Воспроизведение аудиофайлов.
//!
//! Синтез речи остаётся на стороне пользователя: здесь только плеер
//! (см. `docs/DECISIONS.md`). Реализация на `rodio` появится на следующем шаге.

use std::path::Path;

use anyhow::Result;

/// Проиграть аудиофайл.
pub fn play_file(_path: &Path) -> Result<()> {
    Err(anyhow::anyhow!("not implemented"))
}
