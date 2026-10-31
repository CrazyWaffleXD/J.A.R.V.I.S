use std::io::{self, Write};
use std::path::Path;

use jarvis_cmd_open::OpenCommand;
use jarvis_cmd_weather::WeatherCommand;
use jarvis_core::{persona, Command, Config, Context, Dispatcher, Output, Result};
use tracing_subscriber::EnvFilter;

/// Маркер: команда `help` просит REPL напечатать список команд.
const HELP_PLACEHOLDER: &str = "__HELP__";

/// Маркер: команда `voice` просит REPL запустить голосовой режим.
const VOICE_PLACEHOLDER: &str = "__VOICE_LOOP__";

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
        #[cfg(feature = "vosk")]
        {
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

        #[cfg(not(feature = "vosk"))]
        {
            let _ = ctx;
            Err(anyhow::anyhow!(
                "голосовой ввод не скомпилирован. Собери проект с --features vosk и установи libvosk (см. README)"
            ))
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

/// Напечатать список команд. Общий код для REPL и голосового режима:
/// сам `Dispatcher` есть только у REPL, поэтому команда `help` возвращает
/// плейсхолдер, а печать делается здесь.
fn print_help(dispatcher: &Dispatcher) {
    println!("{}", help_text(dispatcher));
}

/// Встроенная команда `voice`: голосовой режим с wake word «Джарвис».
///
/// Сама команда только просит REPL запустить цикл — у неё нет доступа к
/// `Dispatcher`, чтобы выполнять распознанные команды.
struct BuiltinVoiceModeCommand;

#[async_trait::async_trait]
impl Command for BuiltinVoiceModeCommand {
    fn name(&self) -> &'static str {
        "voice"
    }

    fn description(&self) -> &'static str {
        "голосовой режим 'как Алиса': ждёт 'Джарвис', потом команду. Выход: стоп/хватит/выход"
    }

    fn matches(&self, input: &str) -> bool {
        input.trim() == "voice"
    }

    async fn run(&self, _input: &str, ctx: &Context) -> Result<Output> {
        let _ = ctx;

        #[cfg(feature = "vosk")]
        {
            Ok(Output {
                text: VOICE_PLACEHOLDER.to_string(),
                speech: None,
            })
        }

        #[cfg(not(feature = "vosk"))]
        {
            Err(anyhow::anyhow!(
                "голосовой ввод не скомпилирован (--features vosk)"
            ))
        }
    }
}

/// Голосовой режим «как Алиса»: ждёт wake word «Джарвис», записывает
/// отдельную команду, распознаёт её и отдаёт диспетчеру.
///
/// Выход — распознанные «стоп», «хватит», «выход» или «exit».
#[cfg(feature = "vosk")]
async fn run_voice_loop(ctx: &Context, dispatcher: &Dispatcher) -> anyhow::Result<()> {
    let stt = ctx
        .stt
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("STT не настроена (см. README)"))?;

    let stt_config = ctx
        .config
        .stt
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("STT не настроена (см. README)"))?;
    let sample_rate = if stt_config.sample_rate == 0 {
        16_000
    } else {
        stt_config.sample_rate
    };

    let wake = jarvis_wake::WakeWordDetector::new(&stt_config.model_path, sample_rate)?;

    println!("Жду 'Джарвис'... (скажи 'стоп' или Ctrl+C для выхода)");

    loop {
        // 1. Слушаем микрофон, пока не услышим «Джарвис».
        wake.wait_for_wake().await?;
        println!("Слушаю, сэр.");

        // 2. Записываем команду до тишины (но не дольше 10 с).
        let samples = tokio::task::spawn_blocking(move || {
            jarvis_audio::record_until_silence(sample_rate, 1500, 10_000)
        })
        .await??;

        // 3. Распознаём.
        let text = stt.transcribe(&samples, sample_rate).await?;
        let text = text.trim().to_string();

        if text.is_empty() {
            println!("{}", persona::not_understood(""));
            println!("Жду 'Джарвис'...");
            continue;
        }

        println!("Распознано: {text}");

        let lower = text.to_lowercase();

        // 4. Выход из голосового режима.
        if matches!(lower.as_str(), "стоп" | "хватит" | "выход" | "exit") {
            println!("{}", persona::farewell());
            break;
        }

        // 5. Защита от рекурсии: если услышали само wake word (эхо
        //    колонок, ошибка распознавания), команды здесь нет.
        if matches!(lower.as_str(), "джарвис" | "джарвиз" | "jarvis") {
            println!("{}", persona::error("это wake word, нужна команда"));
            println!("Жду 'Джарвис'...");
            continue;
        }

        // 6. Выполняем распознанную команду.
        match dispatcher.dispatch(&text, ctx).await {
            Some(Ok(output)) => {
                if output.text == HELP_PLACEHOLDER {
                    print_help(dispatcher);
                } else if output.text == VOICE_PLACEHOLDER {
                    println!("{}", persona::error("голосовой режим уже запущен"));
                } else {
                    // `__EXECUTE__` и прочие маркеры печатаем как есть:
                    // голосовой режим не должен выполнять вложенные команды.
                    println!("{}", output.text);
                }
            }
            Some(Err(e)) => eprintln!("{}", persona::error(&format!("{e:#}"))),
            None => println!("{}", persona::not_understood(&text)),
        }

        println!("Жду 'Джарвис'...");
    }

    Ok(())
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
#[cfg(feature = "vosk")]
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

/// Без фичи `vosk` голосовой ввод не компилируется — STT всегда `None`.
#[cfg(not(feature = "vosk"))]
fn build_stt(_config: &Config) -> Option<std::sync::Arc<dyn jarvis_core::SpeechToText>> {
    None
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
    dispatcher.register(Box::new(BuiltinVoiceModeCommand));
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
                    print_help(&dispatcher);
                } else if output.text == VOICE_PLACEHOLDER {
                    #[cfg(feature = "vosk")]
                    {
                        if let Err(e) = run_voice_loop(&ctx, &dispatcher).await {
                            eprintln!("{}", persona::error(&format!("{e:#}")));
                        }
                    }
                    // Без фичи `voice` возвращает ошибку, плейсхолдер сюда
                    // не доходит.
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
