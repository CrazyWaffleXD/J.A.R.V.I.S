use crate::command::{Command, Output};
use crate::context::Context;
use crate::error::Result;

/// Реестр команд и точка входа для их выполнения.
#[derive(Default)]
pub struct Dispatcher {
    commands: Vec<Box<dyn Command>>,
}

impl Dispatcher {
    /// Создать пустой диспетчер.
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавить команду в реестр.
    pub fn register(&mut self, cmd: Box<dyn Command>) {
        self.commands.push(cmd);
    }

    /// Все зарегистрированные команды (для справки).
    pub fn commands(&self) -> &[Box<dyn Command>] {
        &self.commands
    }

    /// Найти первую подходящую команду и выполнить её.
    ///
    /// `None` означает, что ни одна команда не взялась за ввод.
    pub async fn dispatch(&self, input: &str, ctx: &Context) -> Option<Result<Output>> {
        for cmd in &self.commands {
            if cmd.matches(input) {
                return Some(cmd.run(input, ctx).await);
            }
        }
        None
    }
}
