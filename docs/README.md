# 📚 BSL Analyzer - Документация

**Версия:** 1.6.0  
**Обновлено:** 2025-08-06  
**Архитектура:** Единая система сборки и версионирования

## 🎯 Быстрый старт

**Новый пользователь?** Начните с [QUICK_START.md](../QUICK_START.md) в корне проекта.

## 📚 Структура документации

### 📖 01. Обзор проекта
- [README.md](01-overview/README.md) - Общий обзор архитектуры
- [unified-concept.md](01-overview/unified-concept.md) - Концепция Unified BSL Index
- [architecture-review.md](01-overview/architecture-review.md) - Архитектурный обзор

### 🔧 02. Компоненты системы
- [unified-index/](02-components/unified-index/) - Unified BSL Index (ядро системы)
- [bsl-parser/](02-components/bsl-parser/) - BSL парсер
- [mcp-server/](02-components/mcp-server/) - MCP сервер для LLM
- [shell-tools/](02-components/shell-tools/) - CLI инструменты

### 📚 03. Руководства
- [development.md](03-guides/development.md) - Гид по разработке
- [integration.md](03-guides/integration.md) - Интеграция с внешними системами
- [llm-usage.md](03-guides/llm-usage.md) - Использование с LLM

### 🔌 04. API документация
- [README.md](04-api/README.md) - API референс

### 🚀 05. Система сборки и версионирования
- [BUILD_SYSTEM.md](05-build-system/BUILD_SYSTEM.md) - Полное описание системы сборки
- Единая система версионирования
- Git workflow автоматизация
- Команды для разработки и релизов

### 📦 06. Публикация и распространение
- [PUBLISHING_GUIDE.md](06-publishing/PUBLISHING_GUIDE.md) - Подробное руководство по публикации
- [PUBLISH_QUICK.md](06-publishing/PUBLISH_QUICK.md) - Быстрая публикация за 5 минут
- [NAMING_ALTERNATIVES.md](06-publishing/NAMING_ALTERNATIVES.md) - Альтернативные названия

### 👨‍💻 07. Разработка
- [AUTOMATION.md](07-development/AUTOMATION.md) - Автоматизация разработки
- [CONTRIBUTING.md](07-development/CONTRIBUTING.md) - Гид для контрибьюторов

### 📜 08. Архив и история
- [CLEANUP_SUMMARY.md](08-legacy/CLEANUP_SUMMARY.md) - История очистки проекта
- [CURRENT_DECISIONS.md](CURRENT_DECISIONS.md) - Текущие архитектурные решения


## 🎯 Рекомендуемый порядок чтения

### Для новых пользователей:
1. [QUICK_START.md](../QUICK_START.md) - быстрый старт
2. [README.md](../README.md) - основная информация  
3. [01-overview/README.md](01-overview/README.md) - обзор архитектуры

### Для разработчиков:
1. [07-development/CONTRIBUTING.md](07-development/CONTRIBUTING.md) - как контрибьютить
2. [05-build-system/BUILD_SYSTEM.md](05-build-system/BUILD_SYSTEM.md) - система сборки
3. [03-guides/development.md](03-guides/development.md) - разработка

### Для публикации:
1. [06-publishing/PUBLISH_QUICK.md](06-publishing/PUBLISH_QUICK.md) - быстрая публикация
2. [06-publishing/PUBLISHING_GUIDE.md](06-publishing/PUBLISHING_GUIDE.md) - подробное руководство

## 🔄 Обновление документации

Документация активно развивается. При изменении функциональности:

1. Обновите соответствующий раздел
2. Проверьте ссылки в других документах
3. Обновите этот README при добавлении новых разделов

## 💡 Помощь и поддержка

- 🐛 **Баги**: [GitHub Issues](https://github.com/your-org/bsl-analyzer/issues)
- 💬 **Вопросы**: [GitHub Discussions](https://github.com/your-org/bsl-analyzer/discussions)
- 📧 **Контакт**: bsl-analyzer-team@example.com