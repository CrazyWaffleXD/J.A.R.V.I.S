use jarvis_core::Dispatcher;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let _dispatcher = Dispatcher::new();

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "jarvis starting");
    println!("jarvis {}", env!("CARGO_PKG_VERSION"));
    println!("TODO: REPL");

    Ok(())
}
