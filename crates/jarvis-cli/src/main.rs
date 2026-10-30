use std::io::{self, Write};

use jarvis_cmd_open::OpenCommand;
use jarvis_core::{Command, Config, Context, Dispatcher, Output, Result};
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

fn help_text(dispatcher: &Dispatcher) -> String {
    dispatcher
        .commands()
        .iter()
        .map(|cmd| format!("{} — {}", cmd.name(), cmd.description()))
        .collect::<Vec<_>>()
        .join("\n")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let mut dispatcher = Dispatcher::new();
    dispatcher.register(Box::new(OpenCommand::new()));
    dispatcher.register(Box::new(BuiltinHelpCommand));

    let ctx = Context {
        config: Config::default(),
    };

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "jarvis starting");
    println!("jarvis {}", env!("CARGO_PKG_VERSION"));

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
            Some(Err(e)) => eprintln!("ошибка: {e:#}"),
            None => println!("не понял: {line}"),
        }
    }

    println!("пока");
    Ok(())
}
