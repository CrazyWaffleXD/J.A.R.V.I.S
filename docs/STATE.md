# State — обновлено 2026-10-31, сессия #2 (сквозной сценарий + cmd-open)

## Готово

- workspace + скелет крейтов (core, cli, platform, audio, commands/*)
- AGENTS.md, SPEC.md, STATE.md, TASKS.md, DECISIONS.md
- LICENSE GPL-3.0 (полный текст)
- rust-toolchain.toml (stable 1.75+)
- cargo fmt / check / clippy -D warnings — чисто
- jarvis-core: новый трейт `Command`
  (`name` / `description` / `matches` / `async run`), `execute` удалён
- jarvis-core: `Output { text, speech }`, `enum Speech { File, Bytes }`
- jarvis-core: `Context { config }` (derive Clone), `Config` (Clone+Default)
- jarvis-core: `Dispatcher` с `Vec<Box<dyn Command>>`,
  `register`, `commands`, `async dispatch` (первая совпавшая команда)
- jarvis-core: реэкспорт `Output`, `Speech`, `Command`, `Dispatcher`,
  `Context`, `Config`, `Result`
- jarvis-cli: REPL на `std::io` (prompt + flush, EOF/exit/quit, пустой ввод)
- jarvis-cli: `BuiltinHelpCommand` (name=help, matches help/?); список
  печатает REPL по плейсхолдеру `__HELP__`, обходя `dispatcher.commands()`
- cmd-open: реальная команда (`open` / `открой`), нормализация URL,
  DDG-поиск для не-URL, вывод «Открываю <url>»
- cmd-weather / cmd-volume / cmd-note: приведены к новой сигнатуре,
  `matches() == false`, `run() -> Err("not implemented")`
- сквозной прогон REPL: open URL / поиск / открой / help / open без
  аргумента / неизвестный ввод / exit — работают как задумано

## В работе

- ничего

## Известные проблемы

- `help` использует плейсхолдер `__HELP__` (осознанно, чтобы не тащить
  Arc/Mutex/Weak); при появлении HTTP API список команд придётся
  формировать иначе
- `webbrowser::open` вызывается синхронно внутри async `run` (короткий
  вызов запуска процесса, на v0.1 приемлемо)

## Следующий шаг

- cmd-weather: нужны reqwest, serde_json и tokio-фичи —
  обсудить и записать в DECISIONS перед добавлением зависимостей

## Важное для следующей сессии

- path-зависимость команд: crates/commands/* →
  ../../jarvis-core (НЕ ../.., там виртуальный манифест)
- jarvis-cli зависит от jarvis-cmd-open по path
- публичный API jarvis-core не менять без DECISIONS.md
- `cargo` в PATH не всегда виден из шелла: добавлять
  `$env:Path = "C:\Users\zam\.cargo\bin;$env:Path"` (Windows)
- погодные/сетевые крейты (reqwest, serde_json) появятся пошагово,
  каждый с записью в DECISIONS
