use std::io::{self, Write};
use std::path::{Path, PathBuf};

use jarvis_cmd_open::OpenCommand;
use jarvis_cmd_weather::WeatherCommand;
use jarvis_core::{persona, Command, Config, Context, Dispatcher, Output, Result};
use tracing_subscriber::EnvFilter;

/// Маркер: команда `help` просит REPL напечатать список команд.
const HELP_PLACEHOLDER: &str = "__HELP__";

/// Маркер: команда `voice` просит REPL запустить голосовой режим.
const VOICE_PLACEHOLDER: &str = "__VOICE_LOOP__";

/// Префикс ответа LLM, означающий «выполни команду», а не «ответь текстом».
///
/// Протокол описан в системном промпте (`jarvis_core::config::
/// DEFAULT_SYSTEM_PROMPT`).
const CMD_PREFIX: &str = "__CMD__:";

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

/// Проиграть звук из `assets/audio`, если файл существует.
///
/// Аудиофайлы опциональны: нет файла или звуковой карты — просто тишина,
/// REPL продолжает работать. `play_file` блокирующая, поэтому уходит в
/// `spawn_blocking`; задачу не ждём, чтобы не задерживать ввод.
fn play_sound(relative_path: &str) {
    let path = Path::new(relative_path);
    if path.exists() {
        let path = path.to_path_buf();
        let _handle = tokio::task::spawn_blocking(move || {
            let _ = jarvis_audio::play_file(&path);
        });
    }
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

/// Встроенная команда `ask`: спросить у LLM напрямую, минуя команды.
///
/// Нужна, когда формулировка похожа на команду: `ask ...` гарантированно
/// уходит в LLM.
struct BuiltinAskCommand;

#[async_trait::async_trait]
impl Command for BuiltinAskCommand {
    fn name(&self) -> &'static str {
        "ask"
    }

    fn description(&self) -> &'static str {
        "спросить у ИИ"
    }

    fn matches(&self, input: &str) -> bool {
        let input = input.trim();
        input == "ask"
            || input == "спроси"
            || input.starts_with("ask ")
            || input.starts_with("спроси ")
    }

    async fn run(&self, input: &str, ctx: &Context) -> Result<Output> {
        let input = input.trim();
        let prompt = input
            .strip_prefix("ask ")
            .or_else(|| input.strip_prefix("спроси "))
            .unwrap_or("")
            .trim();

        if prompt.is_empty() {
            return Err(anyhow::anyhow!("пустой вопрос: ask <вопрос>"));
        }

        let llm = ctx.llm.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "LLM не настроена. Установи MISTRAL_API_KEY и секцию [llm] в config.toml (см. README)"
            )
        })?;

        let answer = llm.complete(prompt, None).await?;
        let answer = answer.trim();
        if answer.is_empty() {
            return Err(anyhow::anyhow!("LLM вернула пустой ответ"));
        }

        // `__CMD__:` отдаём REPL как есть: у команды `ask` нет диспетчера,
        // поэтому команду выполнит вызывающая сторона. Обычный ответ —
        // реплика модели, а не выполненная команда: печатаем текст без
        // приставки «Выполняю, сэр».
        Ok(Output {
            text: answer.to_string(),
            speech: None,
        })
    }
}

/// Встроенная команда `speak`: проиграть указанный аудиофайл.
struct BuiltinSpeakCommand;

#[async_trait::async_trait]
impl Command for BuiltinSpeakCommand {
    fn name(&self) -> &'static str {
        "speak"
    }

    fn description(&self) -> &'static str {
        "проиграть аудиофайл: speak <путь>"
    }

    fn matches(&self, input: &str) -> bool {
        input.trim().starts_with("speak ")
    }

    async fn run(&self, input: &str, _ctx: &Context) -> Result<Output> {
        let path = input.trim().strip_prefix("speak ").unwrap_or("").trim();

        if path.is_empty() {
            return Err(anyhow::anyhow!("пустой путь: speak <файл>"));
        }

        let path = PathBuf::from(path);
        if !path.is_file() {
            return Err(anyhow::anyhow!("файл не найден: {}", path.display()));
        }

        let to_play = path.clone();
        let _handle = tokio::task::spawn_blocking(move || {
            let _ = jarvis_audio::play_file(&to_play);
        });

        Ok(Output {
            text: "Воспроизвожу...".to_string(),
            speech: None,
        })
    }
}

/// Разобрать ответ LLM: `__CMD__:<команда>` → команда, иначе `None`.
fn llm_command(reply: &str) -> Option<&str> {
    reply.trim().strip_prefix(CMD_PREFIX).map(str::trim)
}

/// Выполнить команду, которую запросила LLM.
///
/// `ask` из ответа LLM не выполняем: это снова ушло бы в LLM.
async fn run_llm_command(cmd: &str, ctx: &Context, dispatcher: &Dispatcher) {
    if cmd.is_empty() {
        println!("{}", persona::not_understood(""));
        return;
    }

    if BuiltinAskCommand.matches(cmd) {
        println!("{}", persona::error("не могу выполнить ask из ask"));
        return;
    }

    match dispatcher.dispatch(cmd, ctx).await {
        Some(Ok(output)) => {
            if output.text == HELP_PLACEHOLDER {
                print_help(dispatcher);
            } else if output.text == VOICE_PLACEHOLDER {
                println!(
                    "{}",
                    persona::error(
                        "голосовой режим запускается командой voice, а не из ответа LLM"
                    )
                );
            } else {
                // Вложенные маркеры не выполняем — печатаем как есть.
                println!("{}", output.text);
            }
        }
        Some(Err(e)) => eprintln!("{}", persona::error(&format!("{e:#}"))),
        None => println!("{}", persona::not_understood(cmd)),
    }
}

/// Обработать ответ LLM: запрос на команду выполняем, обычный текст печатаем.
///
/// Обычный ответ модели печатается как есть, без `persona::success`:
/// это реплика ассистента, а не выполненная команда.
async fn handle_llm_reply(reply: &str, input: &str, ctx: &Context, dispatcher: &Dispatcher) {
    let Some(cmd) = llm_command(reply) else {
        if reply.trim().is_empty() {
            println!("{}", persona::not_understood(input));
        } else {
            println!("{}", reply.trim());
        }
        return;
    };

    run_llm_command(cmd, ctx, dispatcher).await;
}

/// Не команда — пробуем LLM. Если LLM не настроена, честно говорим об этом.
async fn answer_with_llm(input: &str, ctx: &Context, dispatcher: &Dispatcher) {
    let Some(llm) = ctx.llm.as_ref() else {
        println!(
            "{}",
            persona::error("не понял команду, и LLM не настроена. См. config.toml")
        );
        return;
    };

    match llm.complete(input, None).await {
        Ok(text) => handle_llm_reply(&text, input, ctx, dispatcher).await,
        Err(e) => eprintln!("{}", persona::error(&format!("{e:#}"))),
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
                } else if let Some(cmd) = llm_command(&output.text) {
                    // `ask` вернул от LLM запрос на команду.
                    run_llm_command(cmd, ctx, dispatcher).await;
                } else {
                    // Прочие маркеры печатаем как есть: голосовой режим
                    // не должен выполнять вложенные команды.
                    println!("{}", output.text);
                }
            }
            Some(Err(e)) => eprintln!("{}", persona::error(&format!("{e:#}"))),
            None => {
                // Не команда — возможно, это вопрос к LLM.
                if let Some(llm) = &ctx.llm {
                    match llm.complete(&text, None).await {
                        Ok(answer) => {
                            handle_llm_reply(&answer, &text, ctx, dispatcher).await;
                        }
                        Err(e) => eprintln!("{}", persona::error(&format!("{e:#}"))),
                    }
                } else {
                    println!("{}", persona::not_understood(&text));
                }
            }
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

/// Построить LLM-провайдера, если есть секция `[llm]` и ключ в окружении.
///
/// Ошибка инициализации не роняет CLI: без LLM работают команды, а
/// свободный текст возвращает «LLM не настроена».
fn build_llm(config: &Config) -> Option<std::sync::Arc<dyn jarvis_core::LlmProvider>> {
    let llm_config = config.llm.as_ref()?;

    match llm_config.provider.trim() {
        "" | "mistral" => match jarvis_llm_mistral::build_mistral(llm_config) {
            Ok(llm) => Some(llm),
            Err(e) => {
                eprintln!("предупреждение: LLM недоступна: {e:#}");
                None
            }
        },
        other => {
            eprintln!(
                "предупреждение: LLM-провайдер '{other}' не поддерживается (пока только mistral)"
            );
            None
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Логи — в stderr, чтобы stdout оставался чистым для REPL и персонажа.
    // `symphonia=warn` глушит info-спам mp3-декодера («estimating duration
    // from bitrate…»): при каждом звуке он писал в stderr несколько строк.
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"))
        .add_directive("symphonia=warn".parse().unwrap());
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();

    let mut dispatcher = Dispatcher::new();
    dispatcher.register(Box::new(OpenCommand::new()));
    dispatcher.register(Box::new(WeatherCommand::new()));
    dispatcher.register(Box::new(BuiltinListenCommand));
    dispatcher.register(Box::new(BuiltinVoiceModeCommand));
    dispatcher.register(Box::new(BuiltinAskCommand));
    dispatcher.register(Box::new(BuiltinSpeakCommand));
    dispatcher.register(Box::new(BuiltinHelpCommand));

    let config = load_config();
    let stt = build_stt(&config);
    let llm = build_llm(&config);
    let ctx = Context { config, stt, llm };

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "jarvis starting");
    println!("{}", persona::greeting());
    play_sound("assets/audio/greeting.mp3");

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
                } else if let Some(cmd) = llm_command(&output.text) {
                    // `ask` вернул от LLM запрос на команду: выполняем её,
                    // а не печатаем строку `__CMD__:...`.
                    run_llm_command(cmd, &ctx, &dispatcher).await;
                } else {
                    // Команда выполнена — подтверждаем голосом, если звук есть.
                    println!("{}", output.text);
                    play_sound("assets/audio/success.mp3");
                }
            }
            Some(Err(e)) => eprintln!("{}", persona::error(&format!("{e:#}"))),
            None => answer_with_llm(line, &ctx, &dispatcher).await,
        }
    }

    println!("{}", persona::farewell());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{llm_command, Dispatcher, OpenCommand, WeatherCommand};

    #[test]
    fn parses_command_reply() {
        assert_eq!(llm_command("__CMD__:открой ютуб"), Some("открой ютуб"));
        assert_eq!(llm_command("  __CMD__: weather \n"), Some("weather"));
        assert_eq!(llm_command("__CMD__:"), Some(""));
    }

    #[test]
    fn plain_answers_are_not_commands() {
        assert_eq!(llm_command("Привет! Чем помочь?"), None);
        assert_eq!(llm_command("попробуй __CMD__:weather"), None);
    }

    /// Команда из ответа LLM должна находиться среди зарегистрированных:
    /// иначе REPL ответит «не понял команду».
    #[test]
    fn command_reply_matches_registered_commands() {
        let mut dispatcher = Dispatcher::new();
        dispatcher.register(Box::new(OpenCommand::new()));
        dispatcher.register(Box::new(WeatherCommand::new()));

        for reply in ["__CMD__:открой ютуб", "__CMD__:weather"] {
            let cmd = llm_command(reply).expect("ответ должен быть командой");
            assert!(
                dispatcher
                    .commands()
                    .iter()
                    .any(|command| command.matches(cmd)),
                "команда из {reply} не найдена"
            );
        }
    }
}
