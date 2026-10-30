# TASKS — v0.1

## v0.1

- [x] workspace + скелет крейтов
- [x] AGENTS.md + SPEC.md + STATE.md + TASKS.md + DECISIONS.md
- [x] LICENSE GPL-3.0 (полный текст)
- [x] трейт Command (name/description/matches/run)
- [x] Dispatcher (struct + new)
- [x] Config (пустой), Context (пустой)
- [x] команды-заглушки: open, weather, volume, note
- [x] jarvis-cli печатает приветствие
- [x] jarvis-core: Output, Speech enum
- [x] jarvis-core: Context с полями (пока только config)
- [x] jarvis-core: Dispatcher::register + dispatch
- [ ] jarvis-core: Config — поля + загрузка TOML
- [x] jarvis-cli: REPL (std::io)
- [ ] jarvis-platform: open_url (Linux/Windows)
- [ ] jarvis-platform: set_volume (Linux pactl / Windows IAudioEndpointVolume)
- [ ] jarvis-audio: play_file (rodio)
- [x] cmd-open
- [ ] cmd-weather (open-meteo или wttr.in)
- [ ] cmd-volume
- [ ] cmd-note (JSON в dirs::data_dir)
- [ ] jarvis-cli: HTTP API (axum, /cmd /speak, X-Token)
- [ ] README финализировать
- [ ] релиз v0.1.0
- [ ] jarvis-cli: rustyline вместо std::io
- [ ] jarvis-cli: история команд в файле
