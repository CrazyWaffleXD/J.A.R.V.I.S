use std::io::{self, Write};
use std::path::Path;

use jarvis_cmd_open::OpenCommand;
use jarvis_cmd_weather::WeatherCommand;
use jarvis_core::{persona, Command, Config, Context, Dispatcher, Output, Result};
use tracing_subscriber::EnvFilter;

/// Маркер: команда `help` просит REPL напечатать список команд.
const HELP_PLACEHOLDER: &str = "__HELP__";

/// Встроенная команда `help`: сам список печатает REPL, у которого есть
/// доступ к зарегистрированным командам.
struct BuiltinHelpCommand;

#[async_trait::async_trait]
impl Command for BuiltinHelpCommand {
    fn name(&self) -> &'static str {
        "help"
    }

    fn description(&self) -> &'static str {
        "список команд"
    }

    fn matches(&self, input: &str) -> bool {
        let input = input.trim();
        input == "help" || input == "?"
    }

    async fn run(&self, _input: &str, _ctx: &Context) -> Result<Output> {
        Ok(Output {
            text: HELP_PLACEHOLDER.to_string(),
            speech: None,
        })
    }
}

/// Встроенная команда `listen`: записать фразу с микрофона и распознать её.
///
/// На этом шаге текст только показывается, а не выполняется как команда.
struct BuiltinListenCommand;

#[async_trait::async_trait]
impl Command for BuiltinListenCommand {
    fn name(&self) -> &'static str {
        "listen"
    }

    fn description(&self) -> &'static str {
        "голосовой ввод (Vosk)"
    }

    fn matches(&self, input: &str) -> bool {
        input.trim() == "listen"
    }

    async fn run(&self, _input: &str, ctx: &Context) -> Result<Output> {
        let stt = ctx.stt.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "STT не настроена. Скачай модель Vosk и пропиши config.stt.model_path (см. README)"
            )
        })?;

        let sample_rate = ctx
            .config
            .stt
            .as_ref()
            .map(|cfg| cfg.sample_rate)
            .unwrap_or(16_000);

        let samples = tokio::task::spawn_blocking(move || {
            jarvis_audio::record_until_silence(sample_rate, 1500, 15000)
        })
        .await??;

        let text = stt.transcribe(&samples, sample_rate).await?;

        if text.trim().is_empty() {
            Ok(Output {
                text: persona::not_understood(""),
                speech: None,
            })
        } else {
            Ok(Output {
                text: format!("Распознано: {}", text.trim()),
                speech: None,
            })
        }
    }
}

fn help_text(dispatcher: &Dispatcher) -> String {
    dispatcher
        .commands()
        .iter()
        .map(|cmd| format!("{} — {}", cmd.name(), cmd.description()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Прочитать `config.toml` из текущего каталога; при ошибке — предупреждение
/// и настройки по умолчанию.
fn load_config() -> Config {
    let path = Path::new("config.toml");
    if !path.exists() {
        return Config::default();
    }

    match Config::load(path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!(
                "предупреждение: не удалось прочитать config.toml: {e:#}; использую настройки по умолчанию"
            );
            Config::default()
        }
    }
}

/// Построить STT, если он настроен. Ошибка инициализации не роняет CLI:
/// команды работают без голосового ввода.
fn build_stt(config: &Config) -> Option<std::sync::Arc<dyn jarvis_core::SpeechToText>> {
    let stt_config = config.stt.as_ref()?;

    match jarvis_stt_vosk::build_vosk(stt_config) {
        Ok(stt) => Some(stt),
        Err(e) => {
            eprintln!("предупреждение: голосовой ввод недоступен: {e:#}");
            None
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Логи — в stderr, чтобы stdout оставался чистым для REPL и персонажа.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let mut dispatcher = Dispatcher::new();
    dispatcher.register(Box::new(OpenCommand::new()));
    dispatcher.register(Box::new(WeatherCommand::new()));
    dispatcher.register(Box::new(BuiltinListenCommand));
    dispatcher.register(Box::new(BuiltinHelpCommand));

    let config = load_config();
    let stt = build_stt(&config);
    let ctx = Context { config, stt };

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "jarvis starting");
    println!("{}", persona::greeting());

    let stdin = io::stdin();
    loop {
        print!("jarvis> ");
        io::stdout().flush()?;

        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "exit" || line == "quit" {
            break;
        }

        match dispatcher.dispatch(line, &ctx).await {
            Some(Ok(output)) => {
                if output.text == HELP_PLACEHOLDER {
                    println!("{}", help_text(&dispatcher));
                } else {
                    println!("{}", output.text);
                }
            }
            Some(Err(e)) => eprintln!("{}", persona::error(&format!("{e:#}"))),
            None => println!("{}", persona::not_understood(line)),
        }
    }

    println!("{}", persona::farewell());
    Ok(())
}
