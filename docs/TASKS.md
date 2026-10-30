# TASKS — v0.1

## v0.1

- [x] workspace + скелет крейтов
- [x] AGENTS.md + SPEC.md + STATE.md + TASKS.md + DECISIONS.md
- [x] LICENSE GPL-3.0 (полный текст)
- [x] трейт Command (name/description/execute)
- [x] Dispatcher (struct + new, без register/dispatch)
- [x] Config (пустой), Context (пустой)
- [x] команды-заглушки: open, weather, volume, note
- [x] jarvis-cli печатает приветствие
- [ ] jarvis-core: Output, Speech enum
- [ ] jarvis-core: Context с полями (config, http, audio)
- [ ] jarvis-core: Dispatcher::register + dispatch
- [ ] jarvis-core: Config — поля + загрузка TOML
- [ ] jarvis-cli: REPL (rustyline)
- [ ] jarvis-platform: open_url (Linux/Windows)
- [ ] jarvis-platform: set_volume (Linux pactl / Windows IAudioEndpointVolume)
- [ ] jarvis-audio: play_file (rodio)
- [ ] cmd-open
- [ ] cmd-weather (open-meteo или wttr.in)
- [ ] cmd-volume
- [ ] cmd-note (JSON в dirs::data_dir)
- [ ] jarvis-cli: HTTP API (axum, /cmd /speak, X-Token)
- [ ] README финализировать
- [ ] релиз v0.1.0
