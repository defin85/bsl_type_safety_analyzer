# 🚀 BSL Analyzer - Единая система сборки и версионирования

## 📋 Обзор системы

BSL Analyzer использует **единую централизованную систему** сборки и версионирования, которая обеспечивает:

- ✅ **Синхронизацию версий** между всеми компонентами проекта
- ⚡ **Автоматизированную сборку** Rust + VSCode расширения  
- 🔄 **Git workflow интеграцию** с husky hooks
- 📦 **Готовые к распространению** .vsix пакеты

## 🎯 Архитектура системы

```
┌─────────────────────────────────────────────────────────┐
│                BSL Analyzer Build System                │
├─────────────────────┬───────────────────────────────────┤
│   Version Management │         Build Pipeline           │
│   (version-sync.js)  │    (build-with-version.js)       │
├─────────────────────┼───────────────────────────────────┤
│ • Cargo.toml        │ • cargo build --release           │
│ • package.json      │ • npm run compile                 │  
│ • extension/pkg.json│ • vsce package                    │
└─────────────────────┴───────────────────────────────────┘
           │                          │
           v                          v
    ┌─────────────┐            ┌─────────────┐
    │ Git Hooks   │            │ Workflows   │
    │ (husky)     │            │ (git-*.js)  │
    └─────────────┘            └─────────────┘
```

## 📚 Команды системы

### 🔢 Управление версиями

```bash
# Автоматическое увеличение версии во всех файлах
npm run version:patch    # 1.3.2 → 1.3.3 (исправления)
npm run version:minor    # 1.3.2 → 1.4.0 (новые функции)
npm run version:major    # 1.3.2 → 2.0.0 (breaking changes)
npm run version:sync     # Синхронизация без изменения версии
```

### 🔨 Сборка проекта

```bash
# Полная сборка с автоматическим версионированием
npm run build:patch      # Version + Rust + Extension (patch)
npm run build:minor      # Version + Rust + Extension (minor)
npm run build:major      # Version + Rust + Extension (major)
npm run build:release    # Только сборка (без версионирования)

# Быстрая пересборка расширения
npm run rebuild:extension # Rust + Extension (с проверкой версий)
```

### 📋 Git Workflows

```bash
# Полный релиз с тегированием
npm run git:release patch   # Версия + сборка + коммит + тег
npm run git:release minor   # Релиз с minor версией
npm run git:release major   # Релиз с major версией

# Разработка без версионирования  
npm run git:dev             # Быстрая сборка для разработки

# Умный коммит с автоматическим версионированием
npm run git:commit "feat: новая функция"     # Автоматически minor
npm run git:commit "fix: исправление"       # Автоматически patch
npm run git:commit "major: breaking change" # Автоматически major

# Только версионирование
npm run git:version patch   # Увеличить версию без сборки
```

### 🧹 Утилиты

```bash
# Проверка и очистка
npm run check:binaries    # Проверка наличия бинарных файлов
npm run cleanup:project   # Очистка временных файлов  
npm run deep-cleanup      # Реорганизация структуры проекта
```

## 🔄 Интеграция с Git Hooks

### Pre-commit Hook
- ✅ Проверяет синхронизацию версий
- 🔨 Автоматически пересобирает расширение
- 📊 Показывает статистику изменений
- 💡 Подсказывает команды для версионирования

### Post-commit Hook  
- 🔍 Анализирует изменения в коммите
- 🔄 Автоматически пересобирает при необходимости
- 📦 Обновляет .vsix файл с актуальной версией

## 📂 Структура файлов версионирования

```
project/
├── Cargo.toml                    # version = "1.3.3"
├── package.json                  # "version": "1.3.3"  
├── vscode-extension/package.json # "version": "1.3.3"
└── scripts/
    ├── version-sync.js           # Синхронизация версий
    ├── build-with-version.js     # Сборка с версионированием
    ├── git-workflow.js           # Git workflows
    └── rebuild-extension.js      # Пересборка расширения
```

## 🎯 Рабочие сценарии

### Сценарий 1: Разработка новой функции
```bash
# 1. Разработка кода
# 2. Тестирование изменений
npm run git:dev

# 3. Коммит с автоматическим версионированием
npm run git:commit "feat: добавлена новая функция анализа"
# → Автоматически увеличит minor версию и создаст коммит
```

### Сценарий 2: Исправление багов
```bash
# 1. Исправление кода
# 2. Тестирование
npm run build:patch  # Увеличит patch версию

# 3. Ручной коммит
git add .
git commit -m "fix: исправлена проблема с парсингом"
```

### Сценарий 3: Релиз версии
```bash
# Полный релиз одной командой
npm run git:release minor

# Включает:
# - Увеличение minor версии  
# - Полную сборку проекта
# - Создание коммита
# - Создание git тега
# - Готовый .vsix файл
```

### Сценарий 4: Экстренное исправление
```bash
# Быстрое исправление и релиз
npm run version:patch        # Увеличить версию
npm run build:release        # Собрать без повторного версионирования
git add . && git commit -m "hotfix: критическое исправление"
git tag v$(node -p "require('./package.json').version")
```

## 🔧 Конфигурация

### Автоматические версии по коммитам
- `feat:`, `feature:` → **minor** версия
- `fix:`, `bugfix:` → **patch** версия  
- `major:`, `breaking:` → **major** версия
- Другие → без версионирования

### Файлы, отслеживаемые для пересборки
- **Rust**: `src/**/*.rs`, `Cargo.toml`, `Cargo.lock`
- **Extension**: `vscode-extension/src/**/*.ts`, `vscode-extension/package.json`

## 📊 Мониторинг версий

Система автоматически проверяет синхронизацию версий:

```bash
# При каждой пересборке показывается:
✅ All versions synchronized: 1.3.3

# При несоответствии:
⚠️  Version mismatch detected:
   Cargo.toml: 1.3.2  
   Extension:  1.3.3
   Root:       1.3.1
💡 Run: npm run version:sync to fix
```

## 🎉 Преимущества единой системы

1. **Нет расхождений версий** - автоматическая синхронизация
2. **Быстрая разработка** - одна команда для полной сборки
3. **Правильное версионирование** - семантические версии по стандарту
4. **Git интеграция** - автоматические hooks и workflows
5. **Готовые релизы** - .vsix файлы с правильными версиями
6. **Простота использования** - интуитивные команды npm run

## 🆘 Устранение проблем

### Версии не синхронизированы
```bash
npm run version:sync  # Исправит автоматически
```

### Сборка не удалась
```bash
npm run check:binaries  # Проверит зависимости
cargo build --release   # Отдельная сборка Rust
```

### Проблемы с расширением
```bash
npm run rebuild:extension  # Пересоберет расширение
npm run cleanup:project    # Очистит временные файлы
```

### Сброс до чистого состояния
```bash
npm run deep-cleanup    # Реорганизует структуру
npm run build:release   # Полная пересборка
```