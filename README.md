# jarvis

Голосовой ассистент с расширяемыми командами. Ядро не зависит от LLM и работает
офлайн; команды — отдельные крейты, которые позже можно собирать в dylib.

- Лицензия: GPL-3.0-only (см. `LICENSE`).
- Платформы v0.1: Linux и Windows.
- Архитектурные решения: `docs/DECISIONS.md`.

## Структура

| Путь | Назначение |
| --- | --- |
| `crates/jarvis-core` | трейты (`Command`, `SpeechToText`, `LlmProvider`), диспетчер, конфиг, контекст |
| `crates/jarvis-cli` | бинарь `jarvis`, REPL, умный выбор команда/LLM |
| `crates/jarvis-platform` | открытие URL, системная громкость |
| `crates/jarvis-audio` | запись с микрофона (воспроизведение — позже) |
| `crates/jarvis-stt-vosk` | офлайн-распознавание речи на Vosk |
| `crates/jarvis-wake` | wake word «Джарвис» (Vosk с грамматикой) |
| `crates/jarvis-llm-mistral` | LLM-провайдер Mistral |
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

Vosk по умолчанию выключен: обычная сборка не требует нативных библиотек.
Голосовой ввод включается фичей `vosk` — см. раздел «Голосовой ввод (Vosk)».

Linux: для записи звука (`cpal`) нужны заголовки ALSA, например
`sudo apt install libasound2-dev`. Для HTTP-запросов (`reqwest` на
`native-tls`) — OpenSSL: `sudo apt install pkg-config libssl-dev`.

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

[llm]
provider = "mistral"
api_key_env = "MISTRAL_API_KEY"
```

Секция `[stt]` необязательна: без неё голосовой ввод просто отключён.
Секция `[llm]` тоже необязательна: без неё работают только команды.

## LLM (Mistral)

Умные ответы на свободный текст. Провайдер опционален: без него `jarvis`
работает как раньше — командами.

**Ключ не хранится в файлах проекта.** В `config.toml` только имя переменной
окружения, сам ключ — в окружении:

```powershell
# Windows (новая сессия терминала подхватит значение)
setx MISTRAL_API_KEY "your_key"
```

```sh
# Linux
export MISTRAL_API_KEY="your_key"
```

Получить ключ: <https://admin.mistral.ai/organization/api-keys>

Включить в `config.toml`:

```toml
[llm]
provider = "mistral"
api_key_env = "MISTRAL_API_KEY"
```

Использование:

```text
jarvis> ask привет
Выполняю, сэр. Здравствуйте! Чем могу помочь?

jarvis> а как дела?
Выполняю, сэр. Спасибо, всё в порядке...
```

Как выбирается между командой и LLM: сначала `matches()` всех команд. Если
команда нашлась (`погода`, `weather`, `open`, `listen`, `voice`, `ask`,
`help`) — выполняется она и в сеть к LLM не уходит. Если не нашлась — текст
отправляется в LLM. Отключить LLM можно, убрав секцию `[llm]` или переменную
окружения: тогда на непонятный ввод будет «не понял команду, и LLM не
настроена».

В голосовом режиме (`voice`) та же логика: распознанная фраза сначала идёт в
команды, а если команды нет — в LLM.

Ограничения: нет истории диалога (каждый вопрос отдельный), нет стриминга,
модель по умолчанию `mistral-small-latest`, таймаут запроса 30 с. Ключ
читается из `api_key_env` при старте; если переменной нет, CLI печатает
предупреждение и работает без LLM.

## Голосовой ввод (Vosk)

Офлайн-распознавание речи через [Vosk](https://alphacephei.com/vosk/).
Нативная библиотека и модель **не входят в репозиторий** и не скачиваются
автоматически — их устанавливает пользователь.

### Обычная сборка (без Vosk)

```sh
cargo build --release
cargo run -p jarvis-cli
```

Работают `open`, `weather`, `help`. Команды `listen` и `voice` видны в
`help`, но возвращают «голосовой ввод не скомпилирован»: нативные библиотеки
и модель для такой сборки не нужны.

### Сборка с Vosk

1. Скачать `libvosk` (см. ниже) и указать `VOSK_LIB_DIR`.
2. Скачать модель (см. ниже) и прописать путь в `config.toml`.
3. Собрать и запустить с фичей `vosk`:

```sh
cargo build --release --features vosk
cargo run -p jarvis-cli --features vosk
```

Фича `vosk` по умолчанию выключена (`default = []`).

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
cargo run -p jarvis-cli --features vosk
```

`VOSK_LIB_DIR` читают `build.rs` крейтов `jarvis-stt-vosk` и `jarvis-wake`.
Можно вместо этого
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
cargo run -p jarvis-cli --features vosk
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

В сборке с `--features vosk`:

```text
jarvis> listen
```

Записывает с микрофона до 1.5 с тишины (но не дольше 15 с) и показывает
распознанный текст: `Распознано: ...`. На этом шаге текст не выполняется как
команда — для выполнения есть голосовой режим `voice` (см. ниже).

Без секции `[stt]` все остальные команды работают, а `listen` возвращает
понятную ошибку:

```text
STT не настроена. Скачай модель Vosk и пропиши config.stt.model_path (см. README)
```

В обычной сборке (без фичи) `listen` отвечает:

```text
голосовой ввод не скомпилирован. Собери проект с --features vosk и установи libvosk (см. README)
```

Если модель не загрузилась (неверный путь, битые файлы), CLI при старте
печатает предупреждение в stderr и продолжает работу без `listen`.

### Голосовой режим «как Алиса»

Требует `--features vosk` и настроенный `config.stt`. Микрофон слушает
постоянно, но реагирует только на своё имя: wake word распознаётся отдельным
распознавателем Vosk с грамматикой `джарвис` / `джарвиз` / `jarvis`, всё
остальное уходит в `[unk]`.

```text
jarvis> voice
Жду 'Джарвис'... (скажи 'стоп' или Ctrl+C для выхода)
# говоришь: "Джарвис"
Слушаю, сэр.
# говоришь: "погода"
Распознано: погода
Выполняю, сэр. Moscow: 7°C, Пасмурно
Жду 'Джарвис'...
# говоришь: "стоп"
До связи, сэр.
```

Выход из режима — распознанные «стоп», «хватит», «выход», «exit» (или Ctrl+C).

**Важно:** «Джарвис» произносится **отдельно** от команды, с паузой.
«Джарвис, погода» одной фразой не сработает: распознаватель wake word
ограничен грамматикой из одних обращений, остальное он отдаёт как `[unk]`,
а команда записывается уже вторым проходом. Это ограничение wake-word-подхода,
такое же, как у Алисы и Siri.

Вторая оговорка: грамматика Vosk работает только со словами из словаря
модели. В `vosk-model-small-ru-0.22` имени «джарвис» может не оказаться —
тогда wake word сработает только на `[unk]`/акустически похожие слова.
Если wake word не ловится, проверьте `vosk-model-small-ru-0.22` и при
необходимости возьмите модель с более широким словарём.

### Ограничение линковки

Крейт `vosk` линкуется с нативной библиотекой на этапе сборки. Фича `vosk`
по умолчанию выключена, поэтому обычная сборка от этого не зависит. При
`--features vosk`:

- без `libvosk.lib` / `libvosk.so` сборка не пройдёт;
- без `libvosk.dll` / `libvosk.so` в `PATH` / `LD_LIBRARY_PATH` собранный
  бинарь не запустится — это ошибка загрузчика ОС, приложение её перехватить
  не может;
- отсутствие модели или конфига перехватывается: CLI стартует с
  предупреждением и работает без голосового ввода.

## Статус

Скелет крейтов, REPL, `open`, `weather`, офлайн-`listen` и голосовой режим
`voice` с wake word «Джарвис» на Vosk (опциональная фича `--features vosk`),
опциональная LLM (Mistral) с умным выбором «команда или ответ ИИ».
`volume` и `note` пока заглушки.
