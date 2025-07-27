# Парсер синтаксис-помощника 1С

## Обзор

Парсер синтаксис-помощника - это интегрированный компонент BSL Type Safety Analyzer, который извлекает и структурирует документацию из архивов справки 1С:Enterprise (.hbk файлы). Компонент полностью портирован с Python проекта `1c-help-parser` на Rust.

## Архитектура компонентов

### 1. HBK Archive Parser (`src/docs_integration/hbk_parser_full.rs`)
- Полный порт Python парсера из `1c-help-parser`
- Читает восстановленные .hbk архивы (требуется предварительная обработка WinRAR)
- Извлекает HTML файлы и метаданные
- Поддерживает работу с файлом `rebuilt.shcntx_ru.zip`

### 2. BSL Syntax Extractor (`src/docs_integration/bsl_syntax_extractor.rs`)
- Парсинг HTML документации с извлечением структурированных данных
- Категоризация элементов:
  - **objects** - объекты BSL (354 элемента)
  - **methods** - методы объектов (15,252 элемента)
  - **properties** - свойства объектов (326 элементов)
  - **functions** - глобальные функции (2,782 элемента)
  - **operators** - операторы языка (6,265 элементов)
- Извлечение метаданных: синтаксис, параметры, возвращаемые значения, примеры

### 3. Chunked Writer (`src/docs_integration/chunked_writer.rs`)
- Разбиение больших объемов данных на управляемые файлы
- Ограничения: 50KB на файл, максимум 50 элементов
- Потоковая запись для эффективной работы с памятью
- Генерация структуры идентичной эталонной из C:\1CProject\Unicom\docs_search

### 4. Chunked Loader (`src/docs_integration/chunked_loader.rs`)
- Эффективная загрузка документации из разбитых файлов
- Кэширование индексов для быстрого доступа
- Поиск по ID элемента и имени объекта
- Загрузка категорий по требованию

### 5. Docs Integration (`src/docs_integration/mod.rs`)
- Единая точка входа для работы с документацией
- Поддержка различных форматов загрузки
- Интеграция с анализатором и LSP сервером

## Улучшенный main_index.json

### Структура индекса

```json
{
  "total_items": 24979,
  "categories": {
    "objects": { "items_count": 354, "chunks_count": 8, "files": [...] },
    "methods": { "items_count": 15252, "chunks_count": 376, "files": [...] },
    // ...
  },
  "statistics": {
    "total_files": 609,
    "total_size_mb": 33.85,
    "average_items_per_file": 41.01,
    "coverage": {
      "html_files_processed": 24979,
      "html_files_total": 24979,
      "coverage_percent": 100.0
    },
    "processing_info": {
      "extraction_time_seconds": 7.85,
      "errors_count": 0,
      "warnings_count": 0
    }
  },
  "version_info": {
    "generator_version": "0.1.0",
    "bsl_version": "8.3.22",
    "platform_version": "8.3.22.1923"
  },
  "item_index": {
    "methods_0": {
      "category": "methods",
      "file": "methods_001.json",
      "object_name": "ДинамическийСписок",
      "title": "ДинамическийСписок.АвтоЗаполнениеДоступныхПолей"
    },
    // ... mapping для всех 24,979 элементов
  }
}
```

### Ключевые особенности

1. **item_index** - прямое сопоставление ID элемента с:
   - Категорией элемента
   - Файлом, содержащим элемент
   - Именем объекта-владельца
   - Полным заголовком элемента

2. **statistics** - детальная статистика обработки:
   - Общий размер и количество файлов
   - Покрытие документации
   - Время обработки

3. **version_info** - информация о версиях для совместимости

## Использование

### Генерация документации

```bash
# Полная обработка всех файлов
cargo run --release --bin process_all_docs

# Тестовая обработка 100 файлов
cargo run --release --bin test_chunked_export
```

### Программный доступ

```rust
use bsl_analyzer::docs_integration::DocsIntegration;

// Загрузка документации
let mut docs = DocsIntegration::new();
docs.load_chunked_documentation("output/docs_search")?;

// Получение информации о методе
if let Some(method) = docs.get_method_info("Сообщить") {
    println!("Параметры: {} обязательных", method.required_params_count);
}

// Автодополнение
let completions = docs.get_completions("Масс");

// Поиск методов
let methods = docs.search_methods("Добавить");
```

## Интеграция с анализатором

Парсер интегрирован как библиотечный компонент, а не отдельные исполняемые файлы:

1. **В анализаторе** - улучшенная верификация вызовов методов
2. **В LSP сервере** - автодополнение и подсказки при наведении
3. **В CLI инструментах** - поиск и анализ документации

## Производительность

- Обработка 24,979 HTML файлов: **7.85 секунд**
- Размер выходных данных: **33.85 MB**
- Количество файлов: **609**
- Среднее количество элементов на файл: **41**

## Структура выходных файлов

```
output/docs_search/
├── main_index.json          # Главный индекс с item_index
├── objects/                 # 354 объекта в 8 файлах
│   ├── objects_001.json
│   ├── objects_002.json
│   └── objects_index.json
├── methods/                 # 15,252 метода в 376 файлах
│   ├── methods_001.json
│   └── methods_index.json
├── properties/              # 326 свойств в 7 файлах
├── functions/               # 2,782 функции в 72 файлах
└── operators/               # 6,265 операторов в 146 файлах
```

## Примеры использования

См. `examples/analyzer_with_docs.rs` для полного примера интеграции.