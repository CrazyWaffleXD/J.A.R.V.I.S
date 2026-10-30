# SPEC — jarvis v0.1

## Что это

Лёгкий модульный голосовой ассистент на Rust: небольшое ядро и команды,
подключаемые как отдельные крейты. Работает офлайн; сеть нужна только тем
командам, которым она действительно нужна (`weather`, `open`).

Не "фулл ИИ": LLM подключается отдельным плагином позже и не входит в ядро.

## Не-цели v0.1

- Собственный синтез речи (TTS остаётся на стороне пользователя, мы только плеер).
- LLM и "понимание" свободного текста внутри ядра.
- Мобильные платформы, macOS, web-клиент.
- Хранение заметок в СУБД.

## Состав v0.1

### Ядро (`jarvis-core`)

- трейт `Command` (`name` / `description` / `execute`);
- `Dispatcher` — регистрация и вызов команд по имени;
- `Context` — состояние сессии (config, http-клиент, handle аудио);
- `Output` / `Speech` — единый результат команды и запрос на озвучку;
- `Config` — загрузка TOML.

Публичный API ядра стабилен: изменения только через `DECISIONS.md`.

### CLI (`jarvis-cli`)

- бинарь `jarvis`;
- REPL на `rustyline`;
- HTTP API на `axum`: `POST /cmd`, `POST /speak`, авторизация по заголовку
  `X-Token`.

### Платформа и аудио

- `jarvis-platform`: `open_url` (Linux/Windows), `set_volume`
  (Linux — `pactl`, Windows — `IAudioEndpointVolume`);
- `jarvis-audio`: `play_file` на `rodio`.

### Команды (`crates/commands/*`)

| Крейт | Команда | Назначение |
| --- | --- | --- |
| `cmd-open` | `open` | открыть URL в браузере по умолчанию |
| `cmd-weather` | `weather` | текущая погода (open-meteo или wttr.in) |
| `cmd-volume` | `volume` | изменить системную громкость |
| `cmd-note` | `note` | заметки в JSON-файле в `dirs::data_dir` |

## Платформы и лицензия

- Целевые платформы v0.1: Linux и Windows.
- Лицензия: GPL-3.0-only.

## Критерии готовности v0.1

- `cargo fmt --all --check`, `cargo check --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings` — чисто.
- Все четыре команды работают из REPL и через HTTP API.
- README описывает установку и запуск.
