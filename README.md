# jarvis

Голосовой ассистент с расширяемыми командами. Ядро не зависит от LLM и работает
офлайн; команды — отдельные крейты, которые позже можно собирать в dylib.

- Лицензия: GPL-3.0-only (см. `LICENSE`).
- Платформы v0.1: Linux и Windows.
- Архитектурные решения: `docs/DECISIONS.md`.

## Структура

| Путь | Назначение |
| --- | --- |
| `crates/jarvis-core` | трейты, диспетчер, конфиг, контекст |
| `crates/jarvis-cli` | бинарь `jarvis`, REPL (пока заглушка) |
| `crates/jarvis-platform` | открытие URL, системная громкость |
| `crates/jarvis-audio` | воспроизведение аудио |
| `crates/commands/cmd-*` | команды: `open`, `weather`, `volume`, `note` |

## Документация

- `AGENTS.md` — правила работы над проектом (в т.ч. для агентов).
- `docs/SPEC.md` — что строим в v0.1.
- `docs/STATE.md` — текущее состояние.
- `docs/TASKS.md` — чеклист v0.1.
- `docs/DECISIONS.md` — архитектурные решения (append-only).

## Сборка

```sh
cargo build --workspace
cargo run -p jarvis-cli
```

Текущий статус: скелет крейтов, команды возвращают `not implemented`.
