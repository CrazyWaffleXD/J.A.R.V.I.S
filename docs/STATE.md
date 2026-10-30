# State — обновлено 2026-10-31, сессия #3 (Vosk STT)

## Готово

- workspace + скелет крейтов (core, cli, platform, audio, stt-vosk,
  commands/*)
- AGENTS.md, SPEC.md, STATE.md, TASKS.md, DECISIONS.md, LICENSE GPL-3.0
- rust-toolchain.toml (stable 1.75+)
- cargo fmt / check / clippy -D warnings — чисто
- jarvis-core: трейт `Command` (`name` / `description` / `matches` /
  `async run`), `Output { text, speech }`, `enum Speech`
- jarvis-core: `Context { config, stt }`, `Dispatcher`
  (`register` / `commands` / `async dispatch`)
- jarvis-core: `persona` — `greeting`, `success`, `error`,
  `not_understood`, `farewell`; пулы `&'static str`, рандом через
  `fastrand`
- jarvis-core: `Config { weather, stt }` + `Config::load` (TOML,
  зависимость `toml = "0.8"`)
- jarvis-core: `stt` — трейт `SpeechToText`
  (`async transcribe(&[f32], u32) -> Result<String>`) и `SttConfig
  { model_path, sample_rate }` (Default: пустой путь, 16000)
- jarvis-cli: REPL на `std::io`; читает `config.toml` из текущего
  каталога (ошибка → предупреждение + defaults); регистрация
  open + weather + listen + help
- jarvis-cli: при старте строит STT через
  `jarvis_stt_vosk::build_vosk`, если есть `config.stt`; ошибка →
  предупреждение в stderr, `Context.stt = None`, CLI продолжает
- jarvis-cli: `listen` — запись `record_until_silence(rate, 1500,
  15000)` в `spawn_blocking`, `transcribe`, вывод «Распознано: …»;
  пустой результат → `persona::not_understood`
- jarvis-cli: логи `tracing` в stderr, stdout чистый
- jarvis-stt-vosk: `VoskStt` (Arc<Model>), `build_vosk`; `Recognizer`
  создаётся внутри `spawn_blocking`; `build.rs` читает `VOSK_LIB_DIR`
- jarvis-audio: `record_until_silence` на cpal 0.15 — default input,
  downmix в моно, RMS-порог `0.01`, опрос каждые 50 мс, линейная
  передискретизация к запрошенной частоте; `play_file` ещё заглушка
- cmd-open: обёрнут в `persona::success`
- cmd-weather: open-meteo + fallback wttr.in (`lang=ru`), таймаут 5 с
- проверено вживую (без модели): `help` показывает `listen`;
  `listen` без `[stt]` печатает «STT не настроена…»; битый путь к
  модели → warning в stderr + CLI работает; `exit` → farewell
- проверено вживую (ранее): greeting / open / not_understood /
  погода через wttr.in

## В работе

- ничего

## Известные проблемы

- Крейт `vosk` линкуется с `libvosk` на этапе сборки:
  `cargo check`/`clippy` проходят без библиотеки (метаданные), но
  `cargo build`/`run` без `libvosk.lib` (Windows) или `libvosk.so`
  (Linux) падают на линковке. Собранный бинарь без DLL/SO в путях
  загрузчика не стартует — это ошибка ОС, перехватить её из
  приложения нельзя. Поэтому «нет libvosk → warn и работаем»
  выполняется только для отсутствующей модели/конфига.
- Windows: официальный `vosk-win64-*.zip` содержит `libvosk.dll`,
  `libvosk.lib` и MinGW-рантайм (`libgcc_s_seh-1.dll` и др.).
  Для сборки нужен `VOSK_LIB_DIR` (или путь в `LIB` MSVC), для
  запуска — каталог в `PATH`.
- `api.open-meteo.com` недоступен в ряде регионов (РФ): погода идёт
  через fallback `wttr.in`.
- wttr.in с `lang=ru`: перевод в `lang_ru[0].value`, `weatherDesc`
  остаётся английским.
- Запись: тишина определяется порогом RMS `0.01`; при высоком шуме
  микрофона запись идёт до `max_ms` (15 с), при очень тихой речи
  возможна ранняя остановка. VAD не используется осознанно.
- Частота устройства (обычно 48000) приводится к `sample_rate`
  линейной интерполяцией — для речи достаточно, но не Hi-Fi.
- `listen` передаёт `config.stt.sample_rate` (fallback 16000) и в
  запись, и в распознавание; модель small-ru обучена на 16 кГц.
- `help` по-прежнему на плейсхолдере `__HELP__`.
- `webbrowser::open` синхронный внутри async `run`.

## Следующий шаг

- голосовой режим: `listen` → распознанный текст в `dispatch`
  (следующая сессия)
- либо cmd-volume (Linux `pactl` / Windows `IAudioEndpointVolume`)
- либо cmd-note (JSON в `dirs::data_dir`)

## Важное для следующей сессии

- `libvosk` и модели Vosk пользователь ставит вручную; в репо их нет
  и автозагрузки нет (см. AGENTS.md, README).
- API `vosk` 0.3.1 отличается от «наивного»: `Model::new` →
  `Option<Model>`, `Recognizer::new(&model, f32)` → `Option`,
  `accept_waveform(&[i16])` без sample_rate,
  `final_result().single()` → `Option<CompleteResultSingle>`, текст в
  поле `.text: &str`.
- `Recognizer` в 0.3.1 формально `Send + Sync`, но создаётся внутри
  `spawn_blocking` (не блокируем async-рантайм, не шарим состояние).
- `Context` не выводит `Debug` автоматически — реализован вручную
  из-за `Arc<dyn SpeechToText>`.
- `config.toml` читается из текущего каталога; `config.toml` в
  `.gitignore`, пример — `config.example.toml`.
- path-зависимости: crates/commands/* → ../../jarvis-core;
  jarvis-stt-vosk → ../jarvis-core
- публичный API jarvis-core не менять без DECISIONS.md
- `cargo` не всегда в PATH из шелла (Windows):
  `$env:Path = "C:\Users\zam\.cargo\bin;$env:Path"`
- при перенаправлении stdout на Windows PowerShell 5.1 пишет файл в
  UTF-16; читать `Get-Content -Raw`, а не как cp866
