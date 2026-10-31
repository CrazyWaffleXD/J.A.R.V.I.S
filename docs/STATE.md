# State — обновлено 2026-10-31, сессия #8 (system prompt и `__CMD__`)

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
  (только при сборке с фичей `vosk`)
- jarvis-cli: Vosk под feature flag — `default = []`,
  `--features vosk` = полная сборка; `listen` зарегистрирован
  всегда, без фичи возвращает «голосовой ввод не скомпилирован»
- jarvis-cli: `listen` — запись `record_until_silence(rate, 1500,
  15000)` в `spawn_blocking`, `transcribe`, вывод «Распознано: …»;
  пустой результат → `persona::not_understood`
- jarvis-wake: `WakeWordDetector` (Arc<Model> + грамматика
  `джарвис`/`джарвиз`/`jarvis`/`[unk]`), `wait_for_wake` слушает
  микрофон через `record_stream` и выходит на partial/result
- jarvis-audio: `record_stream(sample_rate, on_chunk)` — потоковые
  чанки ~100 мс PCM i16, `on_chunk → false` останавливает запись
- jarvis-cli: команда `voice` под фичей `vosk` — REPL получает
  плейсхолдер `__VOICE_LOOP__` и запускает `run_voice_loop`:
  wake word → запись команды (1500/10000) → `transcribe` →
  `dispatch`; выход по «стоп/хватит/выход/exit»
- jarvis-cli: `print_help` вынесен отдельно (help в REPL и в
  голосовом режиме)
- jarvis-core: трейт `LlmProvider` (`async complete(prompt, model)`),
  `Context.llm`, секция `[llm]` в `Config` (`provider`,
  `api_key_env`, по умолчанию `mistral` / `MISTRAL_API_KEY`)
- jarvis-llm-mistral: `MistralClient` (`POST /chat/completions`,
  модель `mistral-small-latest`, таймаут 30 с), `build_mistral`
  читает ключ только из ENV; пустой ответ/HTTP-ошибка → понятная
  ошибка
- jarvis-llm-mistral: модель разрешается по приоритету
  `complete(model)` → `client.model` → `mistral-small-latest`;
  `[llm].model` из конфига прокидывается в клиент (юнит-тест
  `model_priority`)
- jarvis-cli: умный выбор — сначала `dispatcher.dispatch`
  (`matches()`), при `None` запрос в LLM; без LLM — «не понял
  команду, и LLM не настроена»
- jarvis-cli: команда `ask` / `спроси` — принудительный запрос в LLM
- jarvis-cli: в `voice` та же логика: команда → LLM → not_understood
- reqwest: `native-tls` (Schannel на Windows, OpenSSL на Linux)
  вместо `rustls-tls` — системные сертификаты и меньше TLS-сюрпризов
- jarvis-llm-mistral: сетевые ошибки через `anyhow::Context`,
  в REPL печатается вся цепочка (`{e:#}`): таймаут / DNS / TLS /
  hyper, а не только «error sending request»
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
- jarvis-core: `LlmConfig.system_prompt` + `DEFAULT_SYSTEM_PROMPT`
  (роль Джарвиса и правило `__CMD__:`); поле с дефолтом, пустая
  строка отключает промпт; юнит-тесты в `config.rs`
- jarvis-llm-mistral: поле `MistralClient.system_prompt`, в запросе
  первым идёт сообщение `role: "system"`, затем `user`;
  `build_mistral` пробрасывает `cfg.system_prompt`, пустой — не
  отправляется; юнит-тесты `system_prompt_goes_first`,
  `empty_or_missing_system_prompt_is_skipped`
- jarvis-cli: ответ LLM с префиксом `__CMD__:` выполняется диспетчером
  **до** обычного вывода — в REPL, в `voice` и для команды `ask`;
  `ask` отдаёт ответ как есть (у команды нет диспетчера); защита от
  рекурсии «не могу выполнить ask из ask» (ask/спроси); хелперы
  `llm_command` / `run_llm_command` / `handle_llm_reply`; юнит-тесты
  парсинга и маршрутизации
- jarvis-cli: обычный текстовый ответ LLM печатается без
  `persona::success` (нет «Выполняю, сэр.» перед репликой модели) —
  приставка осталась только у выполненных команд
- cmd-open: короткие имена сайтов (`ютуб`/`youtube`, google, яндекс,
  вк, github, telegram, википедия) открывают сайт напрямую, а не поиск
  DDG; юнит-тесты `normalize`
- README: раздел «Настройка LLM (Mistral)» (ключи, `$env:`/`setx`,
  `ministral-8b-latest`, `system_prompt`, протокол `__CMD__`);
  `config.example.toml` — модель бесплатного тарифа
- проверено: `cargo fmt --check`, `cargo check --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings` — чисто;
  `cargo check -p jarvis-cli --features vosk` — чисто; тесты
  `jarvis-core`, `jarvis-llm-mistral`, `jarvis-cmd-open`,
  `jarvis-cli` — 12 passed
- проверено вживую: `открой ютуб` → «Открываю
  https://www.youtube.com» (браузер открылся); `help` и `exit`
  работают; без ключа `ask …` → «LLM не настроена»

## В работе

- ничего

## Известные проблемы

- живой прогон `ask открой ютуб` / `ask какая погода` с реальной
  моделью **не проводился**: на машине нет `MISTRAL_API_KEY`.
  Парсинг `__CMD__:` и маршрутизация до `open`/`weather` покрыты
  юнит-тестами, сам `открой ютуб` проверен вживую.
- модель может вернуть `__CMD__:` с командой, которой нет (опечатка,
  другой язык) — тогда печатается `persona::not_understood`; это
  ожидаемое поведение, а не сбой.
- Крейт `vosk` линкуется с `libvosk` на этапе сборки. Это касается
  только сборки с `--features vosk`: обычные `cargo build` / `run`
  не требуют нативной библиотеки. С фичей:
  `cargo check`/`clippy` проходят без библиотеки (метаданные), но
  `cargo build`/`run` без `libvosk.lib` (Windows) или `libvosk.so`
  (Linux) падают на линковке. Собранный с фичей бинарь без DLL/SO в
  путях загрузчика не стартует — это ошибка ОС, перехватить её из
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
- wake word: грамматика Vosk строится по словарю модели. Если слова
  «джарвис» в словаре нет (`vosk-model-small-ru-0.22`), распознавание
  опирается на `[unk]` и акустически похожие слова — детект может
  быть нестабильным. Лечится моделью с широким словарём.
- wake word: `record_stream` не завершается сам, если устройство
  отвалилось (callback перестаёт вызываться) — цикл ждёт `on_chunk`.
  Для v0.1 принято; Ctrl+C работает.
- голосовой режим: «Джарвис, погода» одной фразой не сработает —
  wake word и команда записываются отдельно (как у Алисы).
- проверено (сессия #7b): `reqwest` на `native-tls` собирается без
  `VOSK_LIB_DIR`, clippy чист; при недоступной сети ошибка печатается
  цепочкой — «не удалось отправить запрос в Mistral: … tunnel error: …
  tcp connect error: … (os error 10061)», а не только
  «error sending request».
- проверено (сессия #7): без `MISTRAL_API_KEY` и с секцией `[llm]` —
  предупреждение при старте, `ask привет` → «LLM не настроена»,
  свободный текст → «не понял команду, и LLM не настроена»; с
  заведомо неверным ключом `ask привет` доходит до Mistral и
  показывает `Mistral API вернул 401 Unauthorized: {"detail":"Invalid
  API Key"}` (проверены URL, заголовок и разбор ошибки); «погода»
  выполняет команду и в LLM не уходит. Живого ответа модели нет —
  валидного ключа на машине не было.
- проверено (сессия #6, без модели и микрофона): `--features vosk`
  собирается с `VOSK_LIB_DIR`; `voice` без фичи отвечает «голосовой
  ввод не скомпилирован (--features vosk)», с фичей и без `[stt]` —
  «STT не настроена (см. README)»; `is_wake` покрыт юнит-тестами
  (`cargo test -p jarvis-wake`). Живой прогон wake word с микрофоном
  и моделью **не проводился** — модели Vosk на машине нет.
- `jarvis-wake/build.rs` дублирует `jarvis-stt-vosk/build.rs`:
  `cargo test -p jarvis-wake` линкуется сам по себе, без
  `jarvis-stt-vosk` в графе, и без своего `link-search` не находит
  `libvosk.lib`.
- `webbrowser::open` синхронный внутри async `run`.

## Следующий шаг

- проверить с реальным ключом: `ask открой ютуб` → `open`,
  `ask какая погода` → `weather`, `ask привет` → текст
- cmd-volume (Linux `pactl` / Windows `IAudioEndpointVolume`)
- либо cmd-note (JSON в `dirs::data_dir`)
- позже: TTS (озвучка ответов) — учесть эхо wake word
- позже: история диалога и стриминг для LLM

## Важное для следующей сессии

- `libvosk` и модели Vosk пользователь ставит вручную; в репо их нет
  и автозагрузки нет (см. AGENTS.md, README).
- «Джарвис» произносится отдельно от команды: wake-распознаватель
  ограничен грамматикой, команда записывается вторым проходом.
- при добавлении TTS эхо станет проблемой: микрофон услышит
  собственную озвучку и может сработать на «Джарвис» в ответе —
  понадобится пауза на время `speech` или отключение wake на TTS.
- `record_stream` дренирует буфер cpal каждый опрос: при
  бесконечном слушании память не растёт (в отличие от накопления в
  `record_until_silence`).
- обычная сборка не требует libvosk; для `listen` нужен
  `--features vosk` + `VOSK_LIB_DIR` (и DLL/SO в пути загрузчика).
- API `vosk` 0.3.1 отличается от «наивного»: `Model::new` →
  `Option<Model>`, `Recognizer::new(&model, f32)` → `Option`,
  `accept_waveform(&[i16])` без sample_rate,
  `final_result()` → `CompleteResult` (не `Option`) и завершает
  поток, поэтому в wake-цикле берём `result().single()` и
  `partial_result().partial`; `new_with_grammar(&model, f32,
  &[&str])` принимает **список фраз**, а не JSON-строку (JSON
  собирает сам крейт).
- `Recognizer` в 0.3.1 формально `Send + Sync`, но создаётся внутри
  `spawn_blocking` (не блокируем async-рантайм, не шарим состояние).
- `Context` не выводит `Debug` автоматически — реализован вручную
  из-за `Arc<dyn SpeechToText>` и `Arc<dyn LlmProvider>`.
- LLM включается только секцией `[llm]`: одного `MISTRAL_API_KEY` без
  секции недостаточно (так задумано — конфиг явный).
- LLM: без истории диалога, без стриминга, таймаут 30 с; ключ
  читается при старте, смена переменной требует перезапуска.
- роутинг «сначала команды»: фраза, начинающаяся со слова команды
  (`погода ...`), уйдёт в команду, а не в LLM — это осознанно.
- `jarvis-llm-mistral` не входит в фичу `vosk`: LLM работает и в
  обычной сборке, нативных зависимостей у неё нет.
- reqwest на `native-tls`: Windows использует системный Schannel,
  Linux — OpenSSL (`pkg-config`, `libssl-dev`). Если сеть падает,
  `{e:#}` показывает причину вплоть до io-ошибки.
- `config.toml` читается из текущего каталога; `config.toml` в
  `.gitignore`, пример — `config.example.toml`.
- `__CMD__:` — строковый протокол из двух мест:
  `jarvis_core::config::DEFAULT_SYSTEM_PROMPT` (что просим у модели) и
  `CMD_PREFIX` в `jarvis-cli` (что распознаём). Менять только вместе,
  иначе модель и парсер разойдутся.
- `ask` не выполняет команды сам (у команды нет `Dispatcher`): он
  возвращает ответ LLM с `__CMD__:` как есть, а выполняет REPL или
  `voice`-цикл. Поэтому логика есть в трёх местах: fallback-LLM,
  `ask`-output и голосовой режим.
- `MISTRAL_API_KEY` на машине нет: сетевые ответы Mistral и живой
  `__CMD__` не проверены.
- Windows PowerShell 5.1: `$OutputEncoding` по умолчанию ASCII, и
  кириллица в stdin пайпа ломается (`открой ютуб` → `?????? ?????`).
  Перед прогоном: `$OutputEncoding = New-Object
  System.Text.UTF8Encoding $false` и
  `[Console]::OutputEncoding = [System.Text.Encoding]::UTF8`.
- `open` теперь знает короткие имена сайтов (`ютуб`, `вк`, …) —
  таблица `known_site` в `cmd-open`; незнакомое слово по-прежнему
  уходит в поиск DuckDuckGo.
- path-зависимости: crates/commands/* → ../../jarvis-core;
  jarvis-stt-vosk → ../jarvis-core
- публичный API jarvis-core не менять без DECISIONS.md
- `cargo` не всегда в PATH из шелла (Windows):
  `$env:Path = "C:\Users\zam\.cargo\bin;$env:Path"`
- при перенаправлении stdout на Windows PowerShell 5.1 пишет файл в
  UTF-16; читать `Get-Content -Raw`, а не как cp866
