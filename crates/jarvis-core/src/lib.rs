//! Ядро jarvis: трейты, диспетчер, конфиг. Публичный API стабилен, менять только через DECISIONS.

pub mod command;
pub mod config;
pub mod context;
pub mod dispatcher;
pub mod error;
pub mod llm;
pub mod persona;
pub mod stt;

pub use command::{Command, Output, Speech};
pub use config::{Config, LlmConfig};
pub use context::Context;
pub use dispatcher::Dispatcher;
pub use error::Result;
pub use llm::LlmProvider;
pub use stt::{SpeechToText, SttConfig};
