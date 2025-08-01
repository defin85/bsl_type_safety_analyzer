# API Reference

Полная документация по всем API BSL Type Safety Analyzer.

## 📚 Разделы

### [Rust API](./rust-api.md)
- Core библиотека API
- UnifiedBslIndex интерфейс
- Парсеры и анализаторы
- Интеграция с проектами

### [CLI API](./cli-api.md)
- Команды командной строки
- Параметры и флаги
- Форматы вывода
- Примеры использования

### [MCP API](./mcp-api.md)
- Model Context Protocol
- JSON-RPC методы
- Интеграция с LLM
- Примеры запросов

## 🚀 Быстрый старт

### CLI
```bash
# Построение индекса
bsl-analyzer index --config /path/to/config

# Поиск типа
bsl-analyzer find "Справочники.Контрагенты"

# Проверка синтаксиса
bsl-analyzer syntaxcheck file.bsl
```

### Rust
```rust
use bsl_analyzer::unified_index::{UnifiedIndexBuilder, UnifiedBslIndex};

// Построение индекса
let builder = UnifiedIndexBuilder::new()?;
let index = builder.build_index(&config_path, "8.3.25")?;

// Поиск типа
if let Some(entity) = index.find_entity("ТаблицаЗначений") {
    println!("Methods: {}", entity.interface.methods.len());
}
```

### MCP
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "find_type",
    "arguments": {
      "type_name": "Документы.Заказ"
    }
  },
  "id": 1
}
```

## 📋 Основные концепции

### BslEntity
Универсальное представление любого BSL типа:
- Платформенные типы (Массив, Структура)
- Объекты конфигурации (Справочники, Документы)
- Формы и их элементы
- Модули с экспортными методами

### UnifiedBslIndex
Единый индекс всех типов с:
- O(1) поиском по имени
- Графом наследования
- Проверкой совместимости типов
- Кешированием для производительности

### Форматы вывода
- **JSON** - для программной обработки
- **Human** - для разработчиков
- **LSP** - для интеграции с IDE
- **SARIF** - для CI/CD