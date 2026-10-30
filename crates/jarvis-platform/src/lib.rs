//! Платформенные интеграции: открытие URL и управление громкостью.
//!
//! Целевые платформы v0.1 — Linux и Windows (см. `docs/DECISIONS.md`).
//! Реализация платформенных вызовов появится на следующих шагах; пока
//! функции возвращают `not implemented`.

use anyhow::Result;

/// Открыть URL в браузере по умолчанию.
pub fn open_url(url: &str) -> Result<()> {
    platform::open_url(url)
}

/// Изменить системную громкость на `delta` (положительное — громче).
pub fn set_volume(delta: f32) -> Result<()> {
    platform::set_volume(delta)
}

#[cfg(target_os = "linux")]
mod platform {
    use anyhow::{anyhow, Result};

    pub fn open_url(_url: &str) -> Result<()> {
        Err(anyhow!("not implemented"))
    }

    pub fn set_volume(_delta: f32) -> Result<()> {
        Err(anyhow!("not implemented"))
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use anyhow::{anyhow, Result};

    pub fn open_url(_url: &str) -> Result<()> {
        Err(anyhow!("not implemented"))
    }

    pub fn set_volume(_delta: f32) -> Result<()> {
        Err(anyhow!("not implemented"))
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod platform {
    use anyhow::{anyhow, Result};

    pub fn open_url(_url: &str) -> Result<()> {
        Err(anyhow!("unsupported platform"))
    }

    pub fn set_volume(_delta: f32) -> Result<()> {
        Err(anyhow!("unsupported platform"))
    }
}
