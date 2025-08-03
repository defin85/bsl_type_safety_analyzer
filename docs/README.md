# BSL Type Safety Analyzer - Документация

**Версия:** 2.0.0  
**Обновлено:** 2025-08-03  
**Архитектура:** Core + Shell с tree-sitter парсером

## 📚 Структура документации

### 🎯 С чего начать?

1. **[Актуальные решения](./CURRENT_DECISIONS.md)** ⭐ - Обязательно к прочтению
2. **[Единая концепция](./01-overview/unified-concept.md)** - Архитектура Core+Shell
3. **[Дорожная карта](../roadmap.md)** - Текущий статус и планы развития

### 📖 Основные разделы

#### 01. Обзор и архитектура
- **[01-overview/](./01-overview/)** - Концепции и архитектурные решения
  - [unified-concept.md](./01-overview/unified-concept.md) ⭐ - Двухуровневая архитектура
  - [architecture-review.md](./01-overview/architecture-review.md) - Итоги мозгового штурма

#### 02. Компоненты системы
- **[02-components/](./02-components/)** - Техническая документация компонентов
  - [unified-index/](./02-components/unified-index/) - Единый индекс BSL типов
  - [mcp-server/](./02-components/mcp-server/) - MCP сервер для LLM
  - [bsl-parser/](./02-components/bsl-parser/) - Tree-sitter парсер

#### 03. Руководства разработчика
- **[03-guides/](./03-guides/)** - Практические руководства
  - [development.md](./03-guides/development.md) - Разработка и контрибуция
  - [integration.md](./03-guides/integration.md) - Интеграция с IDE
  - [llm-usage.md](./03-guides/llm-usage.md) - Использование с LLM

#### 04. API и справочники
- **[04-api/](./04-api/)** - API документация
  - [README.md](./04-api/README.md) - Справочник по API


### 🏗️ Архитектура проекта

```
BSL Type Safety Analyzer v2.0
├── Core System (Heavy) - для LLM
│   ├── UnifiedBslIndex (24,000+ типов)
│   ├── MCP Server (Model Context Protocol)
│   └── Platform Cache (~/.bsl_analyzer/)
└── Shell Tools (Light) - для разработчиков
    ├── CLI валидатор (tree-sitter)
    ├── LSP сервер (минимальный)
    └── IDE интеграция
```

### ⚡ Ключевые технологии

- **Парсер**: tree-sitter (НЕ logos+nom)
- **Хранилище**: UnifiedBslIndex (НЕ HybridDocumentationStorage)
- **LLM интерфейс**: MCP (Model Context Protocol)
- **Кеширование**: Версионное с автоматическим обновлением

### 🔄 Миграция с v1.0

Если вы использовали предыдущую версию:
1. Прочитайте [CURRENT_DECISIONS.md](./CURRENT_DECISIONS.md) для понимания изменений
2. Обновите команды согласно новой архитектуре (см. раздел "Быстрый старт")
3. Удалите старые кеши в temp_* папках, используйте ~/.bsl_analyzer/

## 🚀 Быстрый старт

```bash
# Построение единого индекса (с автокешированием)
cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25"

# Поиск типа в индексе
cargo run --bin query_type -- --name "Справочники.Номенклатура" --config "path/to/config" --show-all-methods

# MCP сервер для Claude/GPT
cargo run --bin mcp_server

# Парсинг платформенной документации (разовая операция)
cargo run --bin extract_platform_docs -- --archive "path/to/1c_docs.zip" --version "8.3.25"
```

## 📋 Приоритет документов

При противоречиях используйте следующий порядок:
1. **[CURRENT_DECISIONS.md](./CURRENT_DECISIONS.md)** ⭐ - Высший приоритет
2. **[roadmap.md](../roadmap.md)** - Актуальные планы
3. **[01-overview/unified-concept.md](./01-overview/unified-concept.md)** - Концепция
4. Остальные документы в 01-04 папках

## 📝 Соглашения документации

- ⭐ Ключевые документы
- ⚠️ Устаревшие документы  
- 📚 Справочные материалы
- 🔧 Технические детали
- Даты в формате YYYY-MM-DD
- Поддержка Mermaid диаграмм