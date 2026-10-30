# jarvis

Голосовой ассистент с расширяемыми командами. Ядро не зависит от LLM и работает
офлайн; команды — отдельные крейты, которые позже можно собирать в dylib.

- Лицензия: GPL-3.0-only (см. `LICENSE`).
- Платформы v0.1: Linux и Windows.
- Архитектурные решения: `docs/DECISIONS.md`.

## Структура

| Путь | Назначение |
| --- | --- |
| `crates/jarvis-core` | трейты (`Command`, `SpeechToText`), диспетчер, конфиг, контекст |
| `crates/jarvis-cli` | бинарь `jarvis`, REPL |
| `crates/jarvis-platform` | открытие URL, системная громкость |
| `crates/jarvis-audio` | запись с микрофона (воспроизведение — позже) |
| `crates/jarvis-stt-vosk` | офлайн-распознавание речи на Vosk |
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

Linux: для записи звука (`cpal`) нужны заголовки ALSA, например
`sudo apt install libasound2-dev`.

## Конфигурация

`jarvis` читает `config.toml` из текущего каталога. Если файла нет — берутся
настройки по умолчанию; если файл не читается — печатается предупреждение и
используются настройки по умолчанию. Пример лежит в `config.example.toml`:

```toml
[weather]
default_city = "Moscow"

[stt]
model_path = "D:/models/vosk-model-small-ru-0.22"
sample_rate = 16000
```

Секция `[stt]` необязательна: без неё голосовой ввод просто отключён.

## Голосовой ввод (Vosk)

Офлайн-распознавание речи через [Vosk](https://alphacephei.com/vosk/).
Нативная библиотека и модель **не входят в репозиторий** и не скачиваются
автоматически — их устанавливает пользователь.

### 1. libvosk

Скачать архив для своей ОС:
<https://github.com/alphacep/vosk-api/releases>

- Windows: `vosk-win64-<версия>.zip` — внутри `libvosk.dll` и `libvosk.lib`.
- Linux: `vosk-linux-x86_64-<версия>.zip` — внутри `libvosk.so`.

**Windows (MSVC).** `libvosk.lib` нужен линковщику при сборке, `libvosk.dll` —
при запуске:

```powershell
# распаковали, например, в D:\vosk\vosk-win64-0.3.45
$env:VOSK_LIB_DIR = "D:\vosk\vosk-win64-0.3.45"   # для сборки
$env:Path = "D:\vosk\vosk-win64-0.3.45;$env:Path" # для запуска
cargo run -p jarvis-cli
```

`VOSK_LIB_DIR` читает `build.rs` крейта `jarvis-stt-vosk`. Можно вместо этого
положить `libvosk.lib` в каталог из переменной `LIB` MSVC, а `libvosk.dll` — в
`PATH` (или рядом с `jarvis.exe`). В архиве также лежат `libgcc_s_seh-1.dll`,
`libstdc++-6.dll`, `libwinpthread-1.dll` — их нужно оставить рядом с DLL.

**Linux.** Положить `libvosk.so` в `/usr/local/lib/` (или `/usr/lib/`) и
обновить кеш:

```sh
sudo cp libvosk.so /usr/local/lib/
sudo ldconfig
```

Либо указать каталог при сборке и запуске:

```sh
export VOSK_LIB_DIR=/path/to/vosk
export LD_LIBRARY_PATH=/path/to/vosk:$LD_LIBRARY_PATH
cargo run -p jarvis-cli
```

### 2. Модель

Скачать `vosk-model-small-ru-0.22` (45 МБ, русский):
<https://alphacephei.com/vosk/models>

Распаковать и прописать путь **к каталогу модели** (не к файлу) в
`config.toml`:

```toml
# Windows
[stt]
model_path = "D:/models/vosk-model-small-ru-0.22"
sample_rate = 16000
```

```toml
# Linux
[stt]
model_path = "/home/user/models/vosk-model-small-ru-0.22"
sample_rate = 16000
```

### 3. Использование

```text
jarvis> listen
```

Записывает с микрофона до 1.5 с тишины (но не дольше 15 с) и показывает
распознанный текст: `Распознано: ...`. На этом шаге текст не выполняется как
команда — связка `listen` → `dispatch` появится позже.

Без секции `[stt]` все остальные команды работают, а `listen` возвращает
понятную ошибку:

```text
STT не настроена. Скачай модель Vosk и пропиши config.stt.model_path (см. README)
```

Если модель не загрузилась (неверный путь, битые файлы), CLI при старте
печатает предупреждение в stderr и продолжает работу без `listen`.

### Ограничение линковки

Крейт `vosk` линкуется с нативной библиотекой на этапе сборки, поэтому:

- без `libvosk.lib` / `libvosk.so` проект не соберётся;
- без `libvosk.dll` / `libvosk.so` в `PATH` / `LD_LIBRARY_PATH` собранный
  бинарь не запустится — это ошибка загрузчика ОС, приложение её перехватить
  не может;
- отсутствие модели или конфига перехватывается: CLI стартует с
  предупреждением и работает без голосового ввода.

## Статус

Скелет крейтов, REPL, `open`, `weather`, офлайн-`listen` на Vosk.
`volume` и `note` пока заглушки.
