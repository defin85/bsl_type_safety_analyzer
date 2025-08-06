# 👨‍💻 Разработка BSL Analyzer

## 📋 Содержание раздела

- [CONTRIBUTING.md](CONTRIBUTING.md) - **Гид для контрибьюторов**
  - Как внести вклад в проект
  - Стандарты кода
  - Процесс код-ревью

- [AUTOMATION.md](AUTOMATION.md) - **Автоматизация разработки** 
  - Git hooks и автоматизация
  - Скрипты для разработки
  - CI/CD настройки

## 🚀 Быстрый старт разработки

### 1. Настройка окружения
```bash
# Клонирование проекта
git clone https://github.com/your-org/bsl-analyzer
cd bsl-analyzer

# Установка зависимостей
cargo build --release
npm install
```

### 2. Основные команды разработки
```bash
# Разработка Rust кода
cargo build                    # Сборка в debug режиме
cargo test                     # Запуск тестов
cargo clippy                   # Линтер

# Разработка VSCode расширения  
npm run rebuild:extension      # Пересборка расширения
npm run build:release          # Полная сборка

# Тестирование
code --install-extension vscode-extension/dist/bsl-type-safety-analyzer-X.X.X.vsix
```

### 3. Workflow разработки
```bash
# 1. Создание ветки для функции
git checkout -b feature/new-awesome-feature

# 2. Разработка с тестированием
npm run build:release
# ... разработка и тестирование ...

# 3. Коммит изменений
git add .
git commit -m "feat: добавлена новая крутая функция"

# 4. Создание Pull Request
git push origin feature/new-awesome-feature
```

## 🔧 Архитектура для разработчиков

```
BSL Analyzer
├── src/
│   ├── unified_index/      # Ядро системы типов
│   ├── bsl_parser/         # BSL парсер
│   ├── mcp_server/         # MCP сервер для LLM
│   ├── lsp/               # Language Server Protocol
│   └── bin/               # CLI утилиты
├── vscode-extension/       # VSCode расширение
├── tests/                 # Тесты и интеграция
└── scripts/               # Скрипты автоматизации
```

## 📚 Стандарты проекта

### Rust код
- Используем `clippy` для линтинга
- Покрытие тестами критических путей
- Документация в формате rustdoc

### TypeScript (VSCode расширение)
- Строгие типы TypeScript
- ESLint для консистентности кода
- Интеграция с LSP сервером

### Документация  
- Markdown с эмодзи для структурирования
- Примеры кода в блоках
- Актуальные ссылки и референсы

## 🤝 Участие в проекте

1. **Issues** - сообщайте о багах и предлагайте функции
2. **Pull Requests** - контрибьютьте код
3. **Discussions** - обсуждайте архитектурные решения
4. **Documentation** - улучшайте документацию

## 📬 Контакты

- GitHub Issues для багов и запросов функций  
- GitHub Discussions для общих вопросов
- Email: bsl-analyzer-team@example.com