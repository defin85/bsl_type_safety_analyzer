# CLAUDE.md

Инструкции для Claude Code при работе с проектом BSL Type Safety Analyzer.

## 🔧 Сборка проекта - ВАЖНО!

**ВСЕГДА используйте готовые npm скрипты для сборки!** Не вызывайте cargo напрямую.

```bash
# ✅ ПРАВИЛЬНО - используйте npm скрипты:
npm run build:rust           # Сборка Rust в release режиме
npm run build:smart          # Умная сборка с кешированием
npm run dev                  # Быстрая dev сборка
npm run rebuild:extension    # Пересборка расширения

# ❌ НЕПРАВИЛЬНО - не используйте cargo напрямую:
# cargo build --release      # НЕ ДЕЛАЙТЕ ТАК!
# CARGO_BUILD_JOBS=4 cargo build  # НЕ ДЕЛАЙТЕ ТАК!
```

**Почему:** npm скрипты автоматически настраивают CARGO_BUILD_JOBS и другие переменные окружения правильно для текущей системы.

## 🎯 Обзор проекта v1.6.0

**BSL Type Safety Analyzer** - корпоративный статический анализатор для 1С:Предприятие BSL, написанный на Rust.

**Статус проекта**: Ready for Publication  
**Текущая версия**: v1.7.1  
**VSCode Extension**: Готово к публикации (~50 MB с бинарниками)

### ✅ Что работает сейчас (v1.7.1):
- **Universal Dev Console v2.0** - полнофункциональная интерактивная консоль (39 функций)
- **Перфектное выравнивание** - меню prompts с идеальным отображением
- **Умная система сборки** - кеширование и watch-режим
- **Unified BSL Type System** - единый индекс всех типов BSL (платформа + конфигурация)
- **Enterprise масштабирование** - оптимизировано для конфигураций с 80,000+ объектов  
- **Автоматическая синхронизация версий** - единые версии во всех компонентах
- **VSCode расширение** - самодостаточное с 27 бинарными инструментами
- **Готовность к публикации** - VS Code Marketplace и GitHub Releases
- **Организованная документация** - структурированная в `docs/`

## 🔧 API Safety Guidelines

### Работа с файлами и документацией
- **Избегать ошибок API** при чтении больших файлов:
  - Использовать `limit` параметр для файлов >2000 строк
  - Проверять содержимое на невалидные Unicode символы
  - При ошибке "no low surrogate in string" - читать файлы частями
- **Безопасные практики:**
  - Использовать Grep для поиска в больших файлах вместо полного чтения
  - При работе с документацией платформы (24,979 элементов) - ограничивать вывод
  - Проверять размер файла перед чтением целиком

### Специфичные рекомендации для проекта
- **Файлы документации 1С**: до 28,944+ токенов (превышают лимит API 25,000)
- **`Global context.html`**: содержит индекс всех глобальных функций - читать по секциям
- **Platform docs extraction**: использовать минимальный console output для 4000+ типов

## Development Commands

### Building and Running

<commands>
  <command-group name="Building">
    <command>
      <description>Build the project</description>
      <code>cargo build</code>
    </command>
    <command>
      <description>Build optimized release version</description>
      <code>cargo build --release</code>
    </command>
  </command-group>

  <command-group name="Unified Index Operations">
    <command>
      <description>Build unified index from configuration (with automatic caching!)</description>
      <code>cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25"</code>
      <performance>
        <first-run>~795ms</first-run>
        <cached>~588ms (25% faster)</cached>
      </performance>
    </command>
    
    <command>
      <description>Build with specific application mode</description>
      <variants>
        <variant mode="ordinary">cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25" --mode ordinary</variant>
        <variant mode="managed" default="true">cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25" --mode managed</variant>
        <variant mode="mixed">cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25" --mode mixed</variant>
      </variants>
    </command>
  </command-group>

  <command-group name="Platform Documentation">
    <command>
      <description>Extract platform documentation (one-time per version)</description>
      <code>cargo run --bin extract_platform_docs -- --archive "path/to/1c_v8.3.25.zip" --version "8.3.25"</code>
      <result>~/.bsl_analyzer/platform_cache/8.3.25.jsonl</result>
    </command>
  </command-group>

  <command-group name="Type Queries">
    <command>
      <description>Query unified index (uses project cache automatically)</description>
      <code>cargo run --bin query_type -- --name "Справочники.Номенклатура" --config "path/to/config" --show-all-methods</code>
    </command>
  </command-group>
</commands>

### 🚀 Unified BSL Index (v1.4.2) - производственная версия

<examples>
  <example-group name="Index Building">
    <example>
      <description>Построение единого индекса из XML конфигурации (автоматически кешируется)</description>
      <code>cargo run --bin build_unified_index -- --config "C:\Config\MyConfig" --platform-version "8.3.25"</code>
      <performance>
        <first-run>~795ms</first-run>
        <cached>~588ms (25% быстрее)</cached>
      </performance>
    </example>
    
    <example>
      <description>Извлечение платформенных типов (один раз для версии)</description>
      <code>cargo run --bin extract_platform_docs -- --archive "path/to/1c_v8.3.25.zip" --version "8.3.25"</code>
      <result>~/.bsl_analyzer/platform_cache/8.3.25.jsonl</result>
    </example>
  </example-group>

  <example-group name="Type Queries">
    <example>
      <description>Поиск платформенного типа</description>
      <code>cargo run --bin query_type -- --name "Массив (Array)" --config "path/to/config" --show-methods</code>
      <expected-output>
Найден тип: Массив (Array)
Тип: Platform
Методы: 15
- Вставить(Индекс: Число, Значение: Произвольный)
- Добавить(Значение: Произвольный)
- Найти(Значение: Произвольный): Число
...
      </expected-output>
    </example>
    
    <example>
      <description>Поиск объекта конфигурации со всеми методами</description>
      <code>cargo run --bin query_type -- --name "Справочники.Номенклатура" --config "path/to/config" --show-all-methods</code>
      <expected-output>
Найден тип: Справочники.Номенклатура
Тип: Configuration
Всего методов (включая унаследованные): 45
Собственные методы: 3
Унаследованные методы: 42 (от СправочникОбъект, ОбъектБД)
      </expected-output>
    </example>
    
    <example>
      <description>Проверка совместимости типов</description>
      <code>cargo run --bin check_type -- --from "Справочники.Номенклатура" --to "СправочникСсылка" --config "path/to/config"</code>
      <expected-output>
✓ Тип "Справочники.Номенклатура" совместим с "СправочникСсылка"
Путь наследования: Справочники.Номенклатура → реализует → СправочникСсылка
      </expected-output>
    </example>
  </example-group>
</examples>

### 🚀 Universal Dev Console v2.0 (НОВОЕ!)

**Полнофункциональная интерактивная консоль разработки - 39 функций в 6 категориях**

```bash
# Запуск интерактивной консоли (основной способ)
npm run interactive

# Альтернативные варианты запуска
./dev.cmd        # Windows: быстрый запуск
./dev.sh         # Linux/Mac: быстрый запуск
```

**🎯 Основные категории:**
- **📦 Сборка и разработка** (8 функций) - dev сборка, smart сборка, watch-режим, пересборка расширения
- **🔄 Версионирование** (6 функций) - patch/minor/major версии, синхронизация, сборка с версией
- **🔧 Разработка и качество** (5 функций) - тесты, clippy, форматирование, проверки, информация о проекте
- **📋 Git операции** (8 функций) - статус, умный коммит, push, workflows, история
- **🚀 Публикация** (7 функций) - упаковка, публикация в VS Code Marketplace/GitHub, проверки
- **⚙️ Утилиты и диагностика** (5 функций) - очистка, watch установка, логи ошибок

**✨ Ключевые возможности:**
- Идеальное выравнивание меню с поддержкой эмодзи
- Система безопасности с подтверждением деструктивных операций  
- Конфигурируемость через `.dev-console-config.json`
- Error logging в `.dev-console-errors.log`
- Graceful shutdown с правильной очисткой ресурсов

### Интеграция с IDE и внешними инструментами
```bash
# MCP сервер для Claude/GPT
cargo run --bin mcp_server

# Базовый LSP сервер (в разработке)
cargo run --bin lsp_server
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test analyzer::semantic_analyzer_integration_test

# Run specific test
cargo test test_method_verification
```

### Development Tools
```bash
# Check code formatting
cargo fmt --check

# Format code
cargo fmt

# Run linter
cargo clippy

# Check for issues
cargo clippy -- -D warnings
```

## 🏗️ Архитектура проекта v1.6.0

**Источники актуальной информации:**
- `docs/` - организованная документация проекта
- `README.md` - основная информация о проекте
- `QUICK_START.md` - быстрый старт для пользователей
- `docs/CURRENT_DECISIONS.md` - архитектурные решения
- `scripts/` - новые инструменты для разработки (v1.6.0)

**Обновлено:** 2025-08-06

### 🆕 Новые возможности v1.6.0

#### 🎯 Интерактивная консоль разработчика (НОВОЕ v1.6.0)
```bash
# Запуск интерактивной консоли с автоматической проверкой зависимостей
npm run interactive
./dev.cmd                    # Windows
./dev.sh                     # Linux/Mac
```

**🆕 Новые возможности:**
- **Автоматическая проверка chokidar** - при запуске watch-режима
- **Однокликовая установка** - предложение установить отсутствующие зависимости
- **Статус-индикаторы** - показ состояния зависимостей в реальном времени
- **Умное меню** - адаптация описаний под текущее состояние системы
- **Ошибкоустойчивость** - никаких больше 'Cannot find module chokidar' ошибок!

#### 👁️ Watch-режим для непрерывной разработки

**Первоначальная настройка:**
```bash
# Установка зависимости для file watcher (однократно)
npm run watch:install        # Автоматическая установка chokidar
# или вручную:
npm install --save-dev chokidar
```

**Команды watch-режима:**
```bash
# Единый watch всех компонентов - автоматическая пересборка при изменениях
npm run watch

# Специализированные watch режимы
npm run watch:rust           # Только Rust файлы (.rs)
npm run watch:extension      # Только TypeScript файлы (.ts)
```

**Smart Watch v2.0 - Возможности с умным кешированием:**

**🆕 Новая интеграция:**
- **🧠 Кеш-интеллект** - Watch + Smart Build = идеальное сочетание!
- **🚀 Мгновенные пересборки** - нет изменений = <1 секунда
- **🎯 Точечная компиляция** - пересборка только того, что реально изменилось
- **📈 Двойная детекция** - file watcher + hash-based change detection

**🎆 Базовые возможности:**
- **Умное отслеживание файлов** - мониторинг .rs и .ts файлов отдельно
- **Очередь сборок** - предотвращение перекрывающихся сборок
- **Обратная связь с кеш-статусом** - показ статуса кеша и операций
- **Восстановление после ошибок** - продолжение работы после неудачных сборок
- **Множественные способы выхода** - Ctrl+C, 'q' + Enter, или kill процесса
- **Graceful shutdown** - корректная очистка ресурсов при выходе

**⚡ Производительность:**
- **Было**: любое изменение = 30-60с полной пересборки
- **Стало**: нет изменений = <1с, есть изменения = только нужное!

#### 🧠 Умное кеширование сборки
```bash
# Команды с кешированием - в 10 раз быстрее традиционных
npm run dev                  # ~2-5s после первой сборки (вместо 30-60s)
npm run build:smart          # Быстрый профиль с кешированием
npm run build:smart:release  # Release сборка с оптимизированным кешем
```

**Принципы работы кеша:**
- **Детекция изменений** - анализ хешей файлов и зависимостей
- **Инкрементальная компиляция** - пересборка только измененного кода
- **Параллельные процессы** - использование всех ядер CPU
- **Профили сборки** - dev/fast/release с разными оптимизациями

#### Core + Shell - Двухуровневая система
```
┌─────────────────────────────────────────────────────────┐
│                   BSL Analyzer v1.2                      │
├─────────────────────────┬───────────────────────────────┤
│      Core (Heavy)       │        Shell (Light)          │
│  LLM-oriented          │      Developer-oriented       │
├─────────────────────────┼───────────────────────────────┤
│  • UnifiedBslIndex      │  • CLI валидатор              │
│  • 500MB+ в памяти ОК   │  • <50ms старт, <10MB памяти  │
│  • MCP Server           │  • tree-sitter парсер         │
│  • База знаний паттернов│  • Offline режим с кешом      │
│  • Семантический граф   │  • Human-friendly вывод       │
└─────────────────────────┴───────────────────────────────┘
```

### 🔧 Ключевые компоненты v1.4.2

**1. Unified Build System** - единая система сборки и версионирования  
**2. UnifiedBslIndex** - единое хранилище всех типов BSL с O(1) поиском  
**3. VSCode Extension** - готовое к публикации расширение  
**4. Documentation System** - организованная документация в `docs/`  
**5. Git Workflow** - прозрачные операции без принудительных hooks

### UnifiedBslIndex - Единый индекс всех BSL типов
**Революционный подход к анализу BSL с автоматическим кешированием**

**Ключевые компоненты:**
- **BslEntity** - универсальное представление любого BSL типа  
- **ConfigurationXmlParser** - прямой парсинг XML без промежуточных отчетов  
- **PlatformDocsCache** - версионное кеширование платформенных типов  
- **ProjectIndexCache** - автоматическое кеширование проектов  
- **UnifiedIndexBuilder** - объединение всех источников в единый индекс

**Производительность (enterprise-масштаб):**
- Индексация 80,000+ объектов: 45-90 секунд
- Загрузка из кеша: 2-3 секунды  
- Поиск типа: <1ms (O(1) HashMap)
- Потребление памяти: ~300MB с LRU кешем
- Поддержка Enterprise конфигураций: ✅ Протестировано

**Структура кеша v2.0:**
```
~/.bsl_analyzer/
├── platform_cache/           # Переиспользуется между проектами
│   ├── 8.3.25.jsonl        # 24,050 типов платформы (~8.5MB)
│   └── 8.3.26.jsonl
└── project_indices/          # Кеши проектов
    └── ProjectName_<hash>/   # Уникальное имя (хеш пути)
        └── 8.3.25/         # Версия платформы
            ├── config_entities.jsonl  # ~5KB
            └── unified_index.json     # ~1KB
```

**Основные API:**

<api-examples>
  <api-method name="find_entity">
    <description>Поиск любой сущности по имени</description>
    <code lang="rust">
// Поиск платформенного типа
let entity = index.find_entity("Массив")?;
let entity = index.find_entity("Array")?; // английский вариант

// Поиск объекта конфигурации  
let entity = index.find_entity("Справочники.Номенклатура")?;
    </code>
  </api-method>
  
  <api-method name="get_all_methods">
    <description>Получение всех методов объекта (включая унаследованные)</description>
    <code lang="rust">
let methods = index.get_all_methods("Справочники.Номенклатура");
// Возвращает HashMap с 45+ методами от СправочникОбъект, ОбъектБД и т.д.

// Проверка наличия метода
if methods.contains_key("Записать") {
    let method = &methods["Записать"];
    println!("Параметры: {:?}", method.parameters);
}
    </code>
  </api-method>
  
  <api-method name="is_assignable">
    <description>Проверка совместимости типов</description>
    <code lang="rust">
// Проверка через интерфейсы
let ok = index.is_assignable("Справочники.Номенклатура", "СправочникСсылка");
assert!(ok); // true - справочник реализует СправочникСсылка

// Проверка несовместимых типов
let ok = index.is_assignable("Число", "Строка");  
assert!(!ok); // false - типы несовместимы
    </code>
  </api-method>
</api-examples>

### 📚 Парсер синтаксис-помощника и документации

**Источник:** `docs/archive/syntax-helper.md`

#### HBK Archive Parser - извлечение документации 1С
- **Компонент:** `src/docs_integration/hbk_parser_full.rs`
- **Источник данных:** Архивы справки `.hbk` / `rebuilt.shcntx_ru.zip` 
- **Результат:** 24,979 элементов BSL документации
- **Производительность:** 7.85 секунд обработки

#### ⚠️ ВАЖНО: Ограничения размера файлов документации
**Файлы документации 1С огромны и часто превышают лимиты API:**
- Размер файлов: до 28,944+ токенов (лимит API: 25,000 токенов)
- `Global context.html` содержит индекс всех глобальных функций
- Отдельные файлы функций (например, `BegOfYear938.html`) с полной документацией

**Рекомендации для анализа больших файлов:**
```bash
# Использовать Read с offset/limit для чтения по частям
Read(file_path="Global context.html", offset=0, limit=1000)

# Использовать Grep для поиска конкретного содержимого  
Grep(pattern="НачалоГода", file_path="Global context.html")
Grep(pattern="BegOfYear", file_path="Global context.html")

# Анализировать структуру по секциям вместо чтения целиком
```

**Архитектурное обоснование:**
- `HbkArchiveParser` спроектирован для поэтапной обработки больших файлов
- Результаты сохраняются в JSONL формате для эффективной работы с большими объемами данных
- Парсер читает файлы по секциям, а не целиком, что критично для файлов размером 100KB+

#### BSL Syntax Extractor - структурированная база знаний
- **Компонент:** `src/docs_integration/bsl_syntax_extractor.rs`
- **Категории элементов:**
  - objects: 354 элемента (Массив, Справочники, etc.)
  - methods: 15,252 элемента (методы объектов)
  - properties: 326 элементов (свойства объектов) 
  - functions: 2,782 элемента (глобальные функции)
  - operators: 6,265 элементов (операторы языка)

#### Структура извлеченной документации
```
~/.bsl_analyzer/ или output/docs_search/
├── main_index.json          # Главный индекс с item_index
├── objects/                 # 354 объекта в 8 файлах
├── methods/                 # 15,252 метода в 376 файлах  
├── properties/              # 326 свойств в 7 файлах
├── functions/               # 2,782 функции в 72 файлах
└── operators/               # 6,265 операторов в 146 файлах
```

#### Интеграция с UnifiedBslIndex
- Типы из документации становятся `BslEntity` в едином индексе
- Автодополнение и подсказки в LSP сервере
- Верификация вызовов методов в анализаторе
- Поддержка поиска методов и свойств

### Актуальные компоненты v2.0

**1. UnifiedBslIndex** - единое хранилище всех типов BSL с O(1) поиском
**2. ConfigurationXmlParser** - прямой парсинг XML без промежуточных отчетов
**3. PlatformDocsCache** - версионное кеширование в ~/.bsl_analyzer/
**4. BSL Syntax Extractor** - извлечение документации через extract_syntax_database()

## 📚 Примеры файлов и структура

### Структура хранения Unified Index (v2.0)
```
~/.bsl_analyzer/
├── platform_cache/                          # Переиспользуется между проектами
│   ├── 8.3.24.jsonl                       # 24,050 типов платформы
│   ├── 8.3.25.jsonl           
│   └── 8.3.26.jsonl
└── project_indices/                        # Индексы проектов
    └── ProjectName_<hash>/                 # Уникальное имя (хеш полного пути)
        ├── 8.3.25/                        # Версия платформы
        │   ├── config_entities.jsonl       # Объекты конфигурации (~5KB)
        │   ├── unified_index.json          # Только индексы конфигурации (~1KB)
        │   └── manifest.json               # Метаданные проекта
        └── 8.3.26/                        # Другая версия платформы
            └── ...
```

### Пример BslEntity
```rust
BslEntity {
    id: BslEntityId("Справочники.Номенклатура"),
    qualified_name: "Справочники.Номенклатура",
    display_name: "Номенклатура",
    entity_type: BslEntityType::Configuration,
    entity_kind: BslEntityKind::Catalog,
    interface: BslInterface {
        methods: {
            "Записать": BslMethod { ... },
            "Прочитать": BslMethod { ... },
            // + все унаследованные от СправочникОбъект
        },
        properties: {
            "Наименование": BslProperty { type: "Строка(150)" },
            "Код": BslProperty { type: "Строка(10)" },
        },
    },
    constraints: BslConstraints {
        parent_types: ["СправочникОбъект"],
        implements: ["СправочникСсылка"],
    },
}
```

## 📊 Текущий статус проекта v1.4.2

### ✅ Готово к публикации (Production Ready)
- **Unified Build System** - полная автоматизация сборки и версионирования
- **VSCode Extension** - самодостаточное расширение с 27 инструментами
- **Documentation Structure** - организованная документация в `docs/`
- **Git Workflow** - прозрачные операции, убраны принудительные hooks
- **Publication Ready** - готово для VS Code Marketplace и GitHub Releases

### 🚧 Core Analysis Features (в разработке)
- **BSL Code Parsing** - tree-sitter парсер (в процессе)
- **Semantic Analysis** - проверка типов и анализ кода
- **LSP Server** - полная реализация Language Server Protocol  
- **MCP Server** - интеграция с LLM через Model Context Protocol

### 🎯 Ближайшие приоритеты
- Завершение BSL парсера для анализа кода
- Улучшение LSP и MCP серверов
- Расширение возможностей анализа

## 📁 Команды разработки

### 🎯 Главная команда разработки (рекомендуется)

```bash
# 🎮 Universal Dev Console v2.0 - интерактивное меню всех команд
npm run interactive

# Быстрые варианты запуска
./dev.cmd        # Windows
./dev.sh         # Linux/Mac
```

**Почему использовать интерактивную консоль:**
- ✅ Все 39 функций в удобном интерфейсе
- ✅ Идеальное выравнивание меню с эмодзи
- ✅ Система безопасности для деструктивных операций
- ✅ Автоматическое логирование ошибок
- ✅ Конфигурируемость и настройки

### 🚀 Команды сборки и разработки (альтернативные)


#### ⚡ Умная сборка с кешированием
```bash
# Основные команды для разработки
npm run dev                  # Быстрая dev сборка с кешированием
npm run build:smart          # Быстрый профиль с кешированием
npm run build:smart:dev      # Dev профиль с автоматическим кешированием
npm run build:smart:release  # Release профиль с оптимизированным кешированием

# Watch режим для непрерывной разработки
npm run watch                # Единый watch режим для всех компонентов
```

#### 🏗️ Система сборки и версионирования
```bash
# Традиционные команды (без кеширования)
npm run build:release        # Полная release сборка
npm run rebuild:extension    # Быстрая пересборка расширения
npm run rebuild:dev          # Dev сборка всех компонентов
npm run rebuild:fast         # Быстрый профиль сборки

# Версионирование и релизы
npm run version:patch        # Увеличить patch версию  
npm run version:minor        # Увеличить minor версию
npm run version:major        # Увеличить major версию
npm run git:release minor    # Полный релиз с minor версией

# Публикация
npm run publish:marketplace  # Опубликовать в VS Code Marketplace
npm run publish:github      # Создать GitHub Release
```

#### 🦀 Rust сборка (разные профили)
```bash
# Профили сборки по скорости (от быстрого к медленному)
cargo build                         # Dev профиль (~40% быстрее release)
cargo build --profile dev-fast      # Компромисс скорость/производительность
cargo build --release              # Максимальная оптимизация

# Специфичные команды
npm run build:rust:dev              # Rust dev сборка
npm run build:rust:fast             # Rust быстрый профиль
npm run build:rust                  # Rust release сборка
```

#### 📊 BSL анализ и индексирование
```bash
# Построение единого индекса
cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25"

# Поиск типов
cargo run --bin query_type -- --name "Справочники.Номенклатура" --config "path/to/config" --show-all-methods

# Извлечение документации платформы
cargo run --bin extract_platform_docs -- --archive "path/to/1c_v8.3.25.zip" --version "8.3.25"
```

#### 🧪 Тестирование и качество кода
```bash
# Rust разработка
cargo test                  # Запуск тестов
cargo clippy                # Линтинг с проверками
cargo fmt                   # Форматирование кода

# Серверы интеграции
cargo run --bin mcp_server   # MCP сервер для LLM
cargo run --bin lsp_server   # LSP сервер (в разработке)
```

## ⚠️ Важные изменения в проекте

### Удалены Git Hooks для прозрачности
**Проблема была:** Git операции висли из-за автоматических сборок во время push
**Решение:** Убраны принудительные pre-commit и post-commit hooks

**Новый workflow разработки:**
```bash
# Прозрачная разработка без автоматики
1. Пишем код
2. npm run rebuild:extension    # когда нужно протестировать
3. git add . && git commit -m "обычный коммит"
4. git push                     # Быстро и прозрачно!
```

### Требования к CLI параметрам
**Все парсеры требуют явного указания исходных файлов:**

✅ **Правильное использование:**
```bash
cargo run --bin extract_platform_docs -- --archive "архив.zip" --version "8.3.25"
cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25"
```

**Преимущества:**
- 🔒 Безопасность - нет скрытых путей
- 📝 Прозрачность - видно какие файлы используются  
- ✅ Валидация - проверка существования файлов
- 📚 Справка - команда `--help` для каждого инструмента

## 🎯 Ключевые достижения проекта

### ✅ Ready for Publication (v1.4.2)
**Проект готов к публикации с полнофункциональной системой сборки:**

**1. Unified Build System**
- Автоматическая синхронизация версий между всеми компонентами
- Единые команды для разработки и релизов
- Git workflow интеграция с умными коммитами

**2. VSCode Extension (Production Ready)**
- Самодостаточное расширение ~50 MB с 27 бинарными инструментами
- Готово для публикации в VS Code Marketplace
- Полная интеграция с BSL анализатором

**3. Организованная документация**
- Структурированная документация в `docs/` с 8 основными разделами
- Руководства для разработчиков, пользователей и публикации
- Четкая навигация и актуальная информация

**4. UnifiedBslIndex - Enterprise масштабирование**
- Поддержка конфигураций с 80,000+ объектов
- O(1) поиск типов, эффективное кеширование
- Единый API для всех типов BSL (платформа + конфигурация)

### 🔧 Технические улучшения
- **Прозрачный Git workflow** - убраны принудительные hooks
- **Explicit CLI parameters** - безопасность и прозрачность
- **Version-aware caching** - переиспользование данных между проектами
- **Direct XML parsing** - без промежуточных текстовых отчетов

## Architecture Overview

The analyzer is structured in several key modules:

### Core Components
- **Parser** (`src/parser/`): Complete BSL lexer and grammar parser with AST generation
- **Analyzer** (`src/analyzer/`): Multi-phase analysis engine including semantic, lexical, and data flow analysis
- **Configuration** (`src/configuration/`): 1C configuration metadata loading and management with integrated parsers
- **Verifiers** (`src/verifiers/`): Method verification and call validation
- **LSP** (`src/lsp/`): Language Server Protocol implementation for editor integration
- **Diagnostics** (`src/diagnostics/`): Error reporting and diagnostic system
- **Documentation Integration** (`src/docs_integration/`): 1C documentation parsing and BSL syntax database

### Key Analysis Phases
1. **Lexical Analysis**: Tokenization and basic syntax validation
2. **Syntax Analysis**: AST construction with grammar validation
3. **Semantic Analysis**: Scope tracking, variable usage, type checking
4. **Method Verification**: Function/procedure call validation and compatibility
5. **Data Flow Analysis**: Variable state tracking and initialization checking

### Configuration Structure
The analyzer understands 1C:Enterprise configuration structure:
- **Modules**: BSL code files with different types (CommonModule, ObjectModule, etc.)
- **Metadata**: Configuration.xml parsing and object relationships
- **Dependencies**: Inter-module dependency tracking and circular dependency detection
- **Enhanced Parsing**: Text configuration reports and XML forms parsing
- **Documentation Integration**: .hbk archives parsing for 1C help system integration

## Development Patterns

### Error Handling
- Use `anyhow::Result<T>` for recoverable errors in most functions
- Use `thiserror` for custom error types in core components
- Collect multiple errors using `ErrorCollector` rather than failing fast

### Testing Approach
- Integration tests are placed alongside modules (e.g., `*_integration_test.rs`)
- Use realistic BSL code samples for testing
- Test both positive and negative cases for analysis rules

### Performance Considerations
- The analyzer uses `rayon` for parallel processing of multiple files
- Parsing uses `nom` combinators for efficient memory usage
- LSP implementation supports incremental parsing for editor responsiveness

### Console Output Design Guidelines
When designing console output for CLI tools, follow these principles:

1. **Use Grouped/Summary Output by Default**
   - Display progress in grouped format (e.g., "Processing 1000/5000 files...")
   - Show summary statistics instead of individual items
   - Use progress indicators for long operations
   - For operations with 1000+ items, show progress every 1000 items, not every 100

2. **Verbose Mode Should Be Optional**
   - Full output only with `--verbose` flag
   - Individual item processing details only when explicitly requested
   - Consider file output for detailed logs
   - Even in verbose mode, limit detailed output to reasonable amounts

3. **Output Limitations**
   - Implement configurable limits for console output
   - Redirect detailed output to files when exceeding thresholds
   - Provide `--output-file` option for full results
   - For platform docs extraction (4000+ types), use minimal console output

4. **Example Implementation Pattern**
   ```rust
   // Good: Grouped progress for large datasets
   if index % 1000 == 0 && index > 0 {
       tracing::info!("Progress: {}/{} files", index, total);
   }
   
   // Better: Final summary only for very large operations
   tracing::info!("Processed {} items in {:.2?}", total, elapsed);
   
   // Bad: Individual item output
   for item in items {
       println!("Processing: {}", item); // Avoid this
   }
   ```

5. **Use Logging Levels Appropriately**
   - `INFO`: Summary and major milestones only
   - `DEBUG`: Progress updates and grouped statistics
   - `TRACE`: Individual item processing details
   - For extract_platform_docs specifically: Use INFO only for start/end messages

6. **Specific Guidelines for Platform Extraction**
   - Extracting 4,916 platform types should show:
     - Start message
     - One progress update at 50%
     - Final summary with counts
   - Avoid showing individual type processing
   - Save detailed type list to cache file, not console

### Code Organization
- Each major component is in its own module with a `mod.rs` that re-exports public APIs
- Analysis phases are designed to be composable and can run independently
- Configuration and metadata handling is centralized in the `configuration` module

## Important Implementation Notes

### BSL Language Support
- Supports both Russian and English BSL keywords
- Handles 1C:Enterprise-specific constructs like export procedures, client/server contexts
- Properly parses 1C configuration metadata (XML format)
- **NEW**: BOM (Byte Order Mark) handling for UTF-8, UTF-16LE, UTF-16BE files
- **NEW**: Multi-encoding support for BSL files (UTF-8, UTF-16, Windows-1251)
- **NEW**: Text configuration reports parsing with multi-encoding support (UTF-16, UTF-8, CP1251)
- **NEW**: XML forms parsing with complete element and attribute extraction
- **NEW**: Documentation integration with 1C help system (.hbk archives)

### Multi-threading
- File analysis can run in parallel using the configured worker count
- LSP server uses async/await with tokio runtime
- Shared state is minimized and protected with appropriate synchronization

### Memory Management
- Uses incremental parsing for LSP to avoid re-parsing entire files
- AST nodes use `Rc` for shared references to reduce memory usage
- Parser supports partial parsing for faster completion and validation

## Common Development Tasks

When adding new analysis rules:
1. Add the rule logic to the appropriate analyzer module
2. Update the `AnalysisResult` structure if new diagnostic types are needed
3. Add integration tests with realistic BSL code examples
4. Update CLI output formatting if new diagnostic categories are added

When extending BSL language support:
1. Update the lexer in `src/parser/lexer.rs` for new tokens
2. Extend the grammar in `src/parser/grammar.rs` for new syntax
3. Add corresponding AST node types in `src/parser/ast.rs`
4. Update semantic analysis to handle new constructs

When modifying configuration handling:
1. Update metadata parsing in `src/configuration/metadata.rs`
2. Extend module discovery logic in `src/configuration/modules.rs`
3. Update dependency tracking in `src/configuration/dependencies.rs`
4. Add validation rules in the main `Configuration::validate()` method

## Modern UnifiedBslIndex Architecture (v2.0)

### Direct XML Configuration Parsing
Modern approach using UnifiedBslIndex without legacy parsers:
- Direct XML parsing through ConfigurationXmlParser
- No need for text configuration reports
- Unified type system with platform documentation integration
- Automatic caching for optimal performance

```rust
// Построение единого индекса из конфигурации
let index = UnifiedIndexBuilder::new()
    .build_index(config_path, "8.3.25")?;

// Поиск типов в едином индексе
let entity = index.find_entity("Справочники.Номенклатура")?;
let methods = index.get_all_methods("Справочники.Номенклатура");
```

### Documentation Integration (`src/docs_integration/`)
**COMPLETE**: Full integration of Python `1c-help-parser` functionality:

#### HBK Archive Parser (`src/docs_integration/hbk_parser_full.rs`)
- Complete 1C documentation archive (.hbk/.shcntx_ru) parsing
- ZIP-based archive reading with file extraction
- Multi-encoding support for HTML content
- Handles 51,000+ HTML documentation files

```rust
let mut parser = HbkArchiveParser::new("rebuilt.shcntx_ru.zip");
parser.open_archive()?;
let content = parser.extract_file_content("syntax/array.html");
```

#### BSL Syntax Extractor (`src/docs_integration/bsl_syntax_extractor.rs`) 
- Extracts complete BSL syntax database from documentation
- Parses methods, properties, functions, operators
- Multi-variant syntax support
- Parameter extraction with types and descriptions

```rust
let mut extractor = BslSyntaxExtractor::new(archive_path);
let database = extractor.extract_syntax_database(None)?;
// database contains: objects, methods, properties, functions, operators
```

#### UnifiedBslIndex Integration (`src/unified_index/`)
**NEW**: Прямая интеграция документации в единый индекс типов:
- Все типы из документации становятся BslEntity
- Единый API для поиска платформенных и конфигурационных типов
- Автоматическое наследование методов и свойств
- Эффективное кеширование и версионирование

```rust
let mut index = UnifiedBslIndex::new();
// Прямое добавление типов из документации
index.add_platform_entities(platform_entities)?;
index.add_config_entities(config_entities)?;
```

### 🔧 Поддержка кодировок и BOM (`src/parser/lexer.rs`)
Улучшенное чтение BSL файлов с автоматическим определением кодировки:
- Автоматическое определение и удаление BOM (UTF-8, UTF-16LE, UTF-16BE)
- Поддержка множественных кодировок с fallback на Windows-1251
- Безопасная обработка границ Unicode символов

```rust
use bsl_analyzer::parser::read_bsl_file;

// Чтение BSL файла с автоматическим определением кодировки и BOM
let content = read_bsl_file("module.bsl")?;
let lexer = BslLexer::new();
let tokens = lexer.tokenize(&content)?; // BOM автоматически удален
```

### 🎯 BSL Parser - Актуальное решение (v1.2.0)
**РЕШЕНИЕ:** `tree-sitter` для BSL Grammar Parser (НЕ logos+nom)

**Источник:** `docs/CURRENT_DECISIONS.md` (2025-08-01)

**Обоснование:**
- Инкрементальный парсинг из коробки
- Готовые binding для множества языков  
- Error recovery для невалидного кода
- Стандарт де-факто для IDE интеграции
- Shell Tools ориентация

**Архитектура:**
- Universal diagnostic output с множественными форматерами (JSON, Human, LSP, SARIF)
- Авто-определение формата вывода по контексту
- Интеграция с UnifiedBslIndex для валидации типов/методов

**⚠️ Устарело:** `logos` + `nom` подход (см. docs/BSL_PARSER_DESIGN.md)

## 📁 Структура примеров и данных

## 📁 Структура проекта и документация

### Основная документация
- `README.md` - основная информация о проекте v1.4.2
- `QUICK_START.md` - быстрый старт для пользователей
- `docs/` - организованная документация с 8 разделами:
  - `01-overview/` - обзор и архитектура
  - `02-components/` - технические компоненты
  - `03-guides/` - руководства пользователей
  - `04-api/` - API документация
  - `05-build-system/` - система сборки
  - `06-publishing/` - публикация и распространение
  - `07-development/` - разработка и контрибьюции
  - `08-legacy/` - архив и история

### Примеры и данные
- `examples/ConfTest/` - тестовая конфигурация для UnifiedBslIndex
- `examples/rebuilt.shcntx_ru.zip` - архив справки 1С для извлечения документации
- `vscode-extension/` - VSCode расширение готовое к публикации

### Кеш и хранилище
- `~/.bsl_analyzer/platform_cache/` - кеш платформенных типов по версиям
- `~/.bsl_analyzer/project_indices/` - индексы проектов с автоматическим кешированием