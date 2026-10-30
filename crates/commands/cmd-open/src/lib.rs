use anyhow::{anyhow, Result};
use jarvis_core::{Command, Context, Output};

/// Команда `open`: открыть URL, сайт или найти запрос в интернете.
#[derive(Debug, Default)]
pub struct OpenCommand;

impl OpenCommand {
    /// Создать команду.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Command for OpenCommand {
    fn name(&self) -> &'static str {
        "open"
    }

    fn description(&self) -> &'static str {
        "открыть URL, сайт или найти в интернете"
    }

    fn matches(&self, input: &str) -> bool {
        input
            .split_whitespace()
            .next()
            .map(|word| {
                let word = word.to_lowercase();
                word == "open" || word == "открой"
            })
            .unwrap_or(false)
    }

    async fn run(&self, input: &str, _ctx: &Context) -> Result<Output> {
        let rest = input
            .trim()
            .split_once(char::is_whitespace)
            .map(|(_, r)| r.trim())
            .filter(|s| !s.is_empty());
        let Some(target) = rest else {
            return Err(anyhow!("укажи URL или запрос: open youtube.com"));
        };

        let url = normalize(target);
        webbrowser::open(&url).map_err(|e| anyhow!("не смог открыть {url}: {e}"))?;

        Ok(Output {
            text: jarvis_core::persona::success(&format!("Открываю {url}")),
            speech: None,
        })
    }
}

/// Привести ввод пользователя к URL: готовый URL, домен или поиск DDG.
fn normalize(target: &str) -> String {
    if target.starts_with("http://") || target.starts_with("https://") {
        target.to_string()
    } else if !target.contains(char::is_whitespace) && target.contains('.') {
        format!("https://{target}")
    } else {
        format!("https://duckduckgo.com/?q={}", urlencoding::encode(target))
    }
}
