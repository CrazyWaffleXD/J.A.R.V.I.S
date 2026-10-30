# State — обновлено 2026-10-31, сессия #1 (bootstrap + fix)

## Готово

- workspace + скелет крейтов (core, cli, platform, audio, commands/*)
- AGENTS.md, SPEC.md, STATE.md, TASKS.md, DECISIONS.md
- LICENSE GPL-3.0 (полный текст)
- rust-toolchain.toml (stable 1.75+)
- cargo fmt / check / clippy -D warnings — чисто
- крейты-заглушки: Command trait (name/description/execute),
  Dispatcher (пустой), Config (пустой), Context (пустой),
  open_url/set_volume/play_file — Err("not implemented")
- команды-заглушки: open, weather, volume, note (все Err)
- jarvis-cli печатает приветствие

## В работе

- ничего

## Известные проблемы

- нет (всё компилится, clippy чист)

## Следующий шаг

- jarvis-core: реализовать Dispatcher::register + dispatch
- jarvis-core: Output, Speech, Context с полями (config, http,
  audio handle)
- после этого — jarvis-cli REPL (rustyline, запись в DECISIONS)

## Важное для следующей сессии

- path-зависимость команд: crates/commands/* →
  ../../jarvis-core (НЕ ../.., там виртуальный манифест)
- публичный API jarvis-core не менять без DECISIONS.md
- тяжёлые крейты (axum, rodio, reqwest, rustyline) появятся
  пошагово, каждый с записью в DECISIONS
