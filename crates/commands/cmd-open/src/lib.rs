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

/// Привести ввод пользователя к URL: готовый URL, известный сайт, домен
/// или поиск DDG.
fn normalize(target: &str) -> String {
    if target.starts_with("http://") || target.starts_with("https://") {
        target.to_string()
    } else if let Some(url) = known_site(target) {
        url.to_string()
    } else if !target.contains(char::is_whitespace) && target.contains('.') {
        format!("https://{target}")
    } else {
        format!("https://duckduckgo.com/?q={}", urlencoding::encode(target))
    }
}

/// Известные сайты по короткому названию.
///
/// LLM охотно возвращает `__CMD__:открой ютуб`, а одно слово без точки
/// уходило бы в поиск. Для популярных сайтов открываем сразу.
fn known_site(target: &str) -> Option<&'static str> {
    match target.trim().to_lowercase().as_str() {
        "ютуб" | "ютьюб" | "ютюб" | "youtube" => Some("https://www.youtube.com"),
        "гугл" | "google" => Some("https://www.google.com"),
        "яндекс" | "yandex" => Some("https://ya.ru"),
        "вк" | "вконтакте" | "vk" => Some("https://vk.com"),
        "гитхаб" | "github" => Some("https://github.com"),
        "телеграм" | "telegram" => Some("https://web.telegram.org"),
        "википедия" | "wikipedia" => Some("https://ru.wikipedia.org"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn known_sites_open_directly() {
        assert_eq!(normalize("ютуб"), "https://www.youtube.com");
        assert_eq!(normalize("YouTube"), "https://www.youtube.com");
        assert_eq!(normalize("вк"), "https://vk.com");
    }

    #[test]
    fn urls_and_domains_are_kept() {
        assert_eq!(normalize("https://example.com/a"), "https://example.com/a");
        assert_eq!(normalize("example.com"), "https://example.com");
    }

    #[test]
    fn the_rest_goes_to_search() {
        assert_eq!(
            normalize("rust было"),
            "https://duckduckgo.com/?q=rust%20%D0%B1%D1%8B%D0%BB%D0%BE"
        );
    }
}
