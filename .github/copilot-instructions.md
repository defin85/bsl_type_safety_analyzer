# Инструкции для GitHub Copilot: как работать с проектом

Краткая выжимка для генерации корректных команд и советов. Используй npm-скрипты, не вызывай cargo напрямую (скрипты правильно настраивают окружение и копируют бинарники в расширение).

## Основные сценарии
- Интерактивная консоль (рекомендуется): `npm run interactive`
- Быстрая dev-сборка с умным кешем: `npm run dev` или `npm run build:smart`
- Watch всех компонентов: `npm run watch` (при необходимости: `npm run watch:install` для chokidar)
- Пересборка расширения VS Code: `npm run rebuild:extension`
- Проверка и копирование бинарников: `npm run check:binaries`, `npm run copy:binaries`

## Rust сборки (через обертки)
- Dev: `npm run build:rust:dev`
- Fast (dev-fast профиль): `npm run build:rust:fast`
- Release: `npm run build:rust:release`
- Универсальная «умная» сборка: `npm run build:smart:dev` | `npm run build:smart:release`

Примечание: прямые вызовы cargo избегать. Если нужен Rust-only watch, подсказать пользователю установить cargo-watch и запускать: `cargo watch -x "build --profile dev-fast"` (или использовать `npm run watch`).

## Git/версии/релизы
- Умный коммит: `npm run git:commit "feat: ..."` | `"fix: ..."` | `"major: ..."`
- Dev-цикл без релиза: `npm run git:dev`
- Полный релиз c версией/тегом/сборкой: `npm run git:release patch|minor|major`
- Синхронизация версий: `npm run version:sync` (или `version:patch|minor|major`)
- Release сборка без изменения версии: `npm run build:release`
- Публикация:
	- VS Code Marketplace: `npm run publish:marketplace`
	- GitHub Releases: `npm run publish:github`

## LSP/расширение VS Code
- Собрать и упаковать расширение: `npm run rebuild:extension` → `npm run package:extension`
- Watch только расширение: `npm run watch:extension`

## CLI-инструменты (Rust binaries)
После сборки бинарники копируются в `vscode-extension/bin`. Ключевые исполняемые:
- Извлечение платформенной документации (однократно на версию):
	- Пример: `cargo run --bin extract_platform_docs -- --archive "path/to/1c_v8.3.25.zip" --version "8.3.25"`
	- Результат кэша: `~/.bsl_analyzer/platform_cache/v8.3.25.jsonl`
- Построение единого индекса типов из конфигурации:
	- Пример: `cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25"`
- Запросы к индексу / проверки совместимости:
	- `cargo run --bin query_type -- --name "Справочники.Номенклатура" --show-all-methods`
	- `cargo run --bin check_type -- --from "Справочники.Номенклатура" --to "СправочникСсылка"`

Подсказка: для больших конфигураций использовать кэши платформы; прямой парсинг Configuration.xml поддерживается. В консоли по умолчанию сводный вывод; подробный — через флаги (см. `--help`).

## Тесты и качество
- Все тесты: `cargo test` (при необходимости вывод: `cargo test -- --nocapture`)
- Форматирование: `cargo fmt`
- Линтер: `cargo clippy` (при необходимости: `cargo clippy -- -D warnings`)

## Частые проблемы
- Watch не запускается: выполнить `npm run watch:install` (установит chokidar)
- Несоответствие версий между файлами: `npm run version:sync`
- Бинарники не найдены в расширении: `npm run copy:binaries` или полная пересборка `npm run rebuild:extension`

## Кратко о структуре
- Unified BSL Index: единый индекс типов (платформа + конфигурация) с кэшированием по версии платформы (`~/.bsl_analyzer/...`).
- Скрипты сборки в `scripts/` автоматизируют cargo, копирование бинарников и упаковку расширения.

