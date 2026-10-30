# State — обновлено 2026-10-31, сессия #2b (persona + cmd-weather)

## Готово

- workspace + скелет крейтов (core, cli, platform, audio, commands/*)
- AGENTS.md, SPEC.md, STATE.md, TASKS.md, DECISIONS.md, LICENSE GPL-3.0
- rust-toolchain.toml (stable 1.75+)
- cargo fmt / check / clippy -D warnings — чисто
- jarvis-core: трейт `Command` (`name` / `description` / `matches` /
  `async run`), `Output { text, speech }`, `enum Speech`
- jarvis-core: `Context { config }`, `Dispatcher`
  (`register` / `commands` / `async dispatch`)
- jarvis-core: `persona` — `greeting`, `success`, `error`,
  `not_understood`, `farewell`; пулы `&'static str`, рандом через
  `fastrand`
- jarvis-core: `Config { weather: WeatherConfig }`,
  `WeatherConfig { default_city }` (Default = "Moscow"), `Deserialize`
  сохранён для будущей загрузки TOML
- jarvis-cli: REPL на `std::io`; регистрация open + weather + help;
  greeting из persona первой строкой, ошибки через `persona::error`,
  неизвестный ввод через `persona::not_understood`, выход через
  `persona::farewell`
- jarvis-cli: логи `tracing` переведены на stderr, чтобы stdout был
  чистым (greeting и ответы REPL)
- cmd-open: обёрнут в `persona::success` («… Открываю <url>»)
- cmd-weather: open-meteo — geocoding
  (`geocoding-api.open-meteo.com`) + forecast
  (`api.open-meteo.com`), WMO-коды → русские описания, город из
  аргумента или `config.weather.default_city`
- cmd-weather: fallback на `https://wttr.in/{city}?format=j1`, если
  open-meteo упал (сеть/блокировка); HTTP-таймаут 5 с на запрос у обоих
  провайдеров
- проверено вживую: greeting / open / help / not_understood / farewell,
  погода через wttr.in (`Moscow: 7°C, Overcast`, `Москва: 7°C, Overcast`)

## В работе

- ничего

## Известные проблемы

- `api.open-meteo.com` недоступен в ряде регионов (РФ, блокировка
  хоста): forecast таймаутит, поэтому в этом окружении погода идёт
  через fallback `wttr.in` (работает). Geocoding open-meteo при этом
  доступен.
- wttr.in отдаёт `weatherDesc` на английском (`Overcast`, `Light rain`),
  без `lang=ru`; формат ответа это не меняет.
- `weather Atlantis` не даёт «город не найден»: open-meteo знает
  Atlantis (ЮАР), поэтому по текущей логике вернётся погода для него.
  Ошибка «город не найден» возникает только при пустом `results`.
- Температура округляется при выводе (`{temp:.0}` → `5°C`); сырое
  значение open-meteo — `f64` (`5.3`), поэтому фактическая цифра может
  отличаться от показанной на десятые.
- Ошибки HTTP-шагов помечены контекстом `geocoding` / `forecast`
  (`.context(...)` из anyhow), в REPL видна вся цепочка, например
  `forecast: error sending request …: tcp connect error …`
- `help` по-прежнему на плейсхолдере `__HELP__` (осознанно, без
  Arc/Mutex/Weak)
- `webbrowser::open` синхронный внутри async `run` (короткий вызов)

## Следующий шаг

- cmd-volume (Linux `pactl` / Windows `IAudioEndpointVolume`)
- cmd-note (JSON в `dirs::data_dir`)

## Важное для следующей сессии

- `reqwest` подключён с `default-features = false` и
  `features = ["json", "rustls-tls"]` — не добавлять native-tls/openssl
- город для weather берётся из `config.weather.default_city`
  (default "Moscow"); аргумент команды перекрывает конфиг
- weather: сначала open-meteo, при любой ошибке — wttr.in
  (`?format=j1`); у обоих клиентов таймаут 5 с
- path-зависимость команд: crates/commands/* → ../../jarvis-core
- jarvis-cli зависит от jarvis-cmd-open и jarvis-cmd-weather по path
- публичный API jarvis-core не менять без DECISIONS.md
- `cargo` не всегда в PATH из шелла (Windows):
  `$env:Path = "C:\Users\zam\.cargo\bin;$env:Path"`
- при перенаправлении stdout на Windows Rust пишет в OEM-кодировке
  (cp866) — читать вывод с учётом этого
