# BSL Type Safety Analyzer - Документация

**Версия:** 1.0.0  
**Обновлено:** 2025-08-01

## 📚 Структура документации

### 🎯 С чего начать?

1. **[Единая концепция проекта](./UNIFIED_CONCEPT.md)** - обязательно к прочтению
2. **[Дорожная карта](../roadmap.md)** - текущий статус и планы
3. **[Руководство по интеграции](./INTEGRATION_APPROACHES.md)** - как использовать проект

### 📖 Основные разделы

#### Обзор и архитектура
- **[UNIFIED_CONCEPT.md](./UNIFIED_CONCEPT.md)** ⭐ - Главный документ: двухуровневая архитектура Core+Shell
- **[ARCHITECTURE_REVIEW_SUMMARY.md](./ARCHITECTURE_REVIEW_SUMMARY.md)** - Итоги архитектурного мозгового штурма
- **[REQUIREMENTS_V2.md](./REQUIREMENTS_V2.md)** - Актуальные требования к системе

#### Компоненты системы
- **[UNIFIED_INDEX_ARCHITECTURE.md](./UNIFIED_INDEX_ARCHITECTURE.md)** - Архитектура единого индекса типов
- **[MCP_SERVER_DESIGN.md](./MCP_SERVER_DESIGN.md)** - Дизайн MCP сервера для LLM
- **[BSL_PARSER_DESIGN.md](./BSL_PARSER_DESIGN.md)** ⚠️ - Устарел, см. REQUIREMENTS_V2 (tree-sitter)

#### Руководства разработчика
- **[INTEGRATION_APPROACHES.md](./INTEGRATION_APPROACHES.md)** - Варианты интеграции с IDE и CLI
- **[LLM_USAGE_GUIDE.md](./LLM_USAGE_GUIDE.md)** - Как использовать с Claude/GPT
- **[BSL_GRAMMAR_DEVELOPMENT.md](./BSL_GRAMMAR_DEVELOPMENT.md)** - Разработка BSL грамматики

#### Техническая документация
- **[api_reference.md](./api_reference.md)** - API reference
- **[DOCUMENTATION_INTEGRATION.md](./DOCUMENTATION_INTEGRATION.md)** - Интеграция документации 1С
- **[HYBRID_STORAGE_ARCHITECTURE.md](./HYBRID_STORAGE_ARCHITECTURE.md)** - Архитектура хранилища

### ⚠️ Важные замечания

1. **Актуальная архитектура парсера**: Используем **tree-sitter**, НЕ logos+nom (см. roadmap.md)
2. **Приоритет документов**: UNIFIED_CONCEPT.md > roadmap.md > остальные
3. **Устаревшие документы**: 
   - BSL_PARSER_DESIGN.md (описывает logos+nom вместо tree-sitter)
   - REFACTORING_PLAN.md (старый план рефакторинга)
   - architecture.md, API.md (дублируют более новые версии)

### 🔄 История изменений

- **2025-08-01**: Мозговой штурм, переход на двухуровневую архитектуру Core+Shell
- **2025-07-30**: Начальные документы с фокусом на разработчиков
- **2025-07-28**: Первые версии парсеров и документации

## 🚀 Быстрый старт

```bash
# Установка
cargo install bsl-analyzer

# Построение индекса
bsl-analyzer index --config "path/to/1c/config"

# Поиск типа
bsl-analyzer find "Справочники.Контрагенты"

# MCP сервер для LLM
bsl-analyzer mcp-server
```

## 📝 Соглашения

- Документы датируются в формате YYYY-MM-DD
- Устаревшие версии помечаются ⚠️
- Главные документы помечаются ⭐
- Используем Markdown с поддержкой Mermaid диаграмм