# 🤖 BSL Analyzer - Автоматизация сборки расширения

## 🚀 Быстрый старт

### Ручная пересборка
```bash
# Полная пересборка с упаковкой
npm run rebuild:extension

# Быстрая пересборка без упаковки (для разработки)
npm run rebuild:quick
```

### Автоматическая пересборка
После установки Husky расширение **автоматически пересобирается** при коммитах, которые содержат изменения в:
- `src/**/*.rs` - исходный код Rust
- `Cargo.toml`, `Cargo.lock` - зависимости Rust
- `vscode-extension/src/**/*.ts` - код расширения TypeScript
- `vscode-extension/package.json` - настройки расширения

## 📋 Доступные команды

| Команда | Описание | Время выполнения |
|---------|-----------|------------------|
| `npm run rebuild:extension` | 🔄 Полная пересборка + упаковка .vsix | ~3-5 минут |
| `npm run rebuild:quick` | ⚡ Быстрая пересборка без упаковки | ~2-3 минуты |
| `npm run build:rust` | 🦀 Только сборка Rust бинарников | ~2-3 минуты |
| `npm run build:extension` | 📝 Только компиляция TypeScript | ~10-20 секунд |
| `npm run copy:binaries` | 📁 Копирование бинарников в расширение | ~1 секунда |

## 🔧 Git Hooks автоматизация

### Установленные hooks:
- **post-commit**: Автоматически пересобирает расширение при изменениях в исходном коде

### Логика работы post-commit hook:
1. ✅ Проверяет, изменились ли файлы Rust или TypeScript в последнем коммите
2. 🦀 При изменении Rust: пересобирает бинарники → копирует в расширение
3. 📝 При изменении TypeScript: компилирует код расширения
4. 📦 Упаковывает финальный .vsix файл
5. 🧹 Удаляет старые версии пакетов

### Пример вывода hook:
```
🦀 Rust source files changed - extension rebuild needed
🔄 Starting automatic extension rebuild...
Building Rust binaries...
Copying binaries to extension...
Compiling TypeScript...
Packaging extension...
✅ Extension successfully rebuilt: bsl-analyzer-1.3.1.vsix
📋 To install: Ctrl+Shift+P → Extensions: Install from VSIX
```

## 🛠️ Структура автоматизации

```
bsl_type_safety_analyzer/
├── package.json                    # npm скрипты и Husky конфигурация
├── scripts/
│   └── rebuild-extension.js        # Основной скрипт пересборки
├── .husky/
│   ├── pre-commit                  # Hook перед коммитом (пустой)
│   └── post-commit                 # Hook после коммита (автопересборка)
├── vscode-extension/
│   ├── bin/                        # Автоматически обновляемые бинарники
│   └── dist/                       # Результаты сборки расширения
│       └── bsl-analyzer-1.3.1.vsix
└── target/release/                 # Rust бинарники
    ├── lsp_server.exe
    ├── bsl-analyzer.exe
    └── ...
```

## 🎯 Workflow разработки

### 1. Разработка Rust кода:
```bash
# Внесите изменения в src/
git add src/
git commit -m "feat: улучшение анализатора"
# → Автоматически пересобирается расширение
```

### 2. Разработка расширения VS Code:
```bash
# Внесите изменения в vscode-extension/src/
git add vscode-extension/
git commit -m "feat: новая команда в расширении"
# → Автоматически пересобирается расширение
```

### 3. Быстрое тестирование без коммита:
```bash
npm run rebuild:quick
# → Пересборка без упаковки .vsix (быстрее)
```

## 🚨 Устранение неполадок

### Проблема: Hook не срабатывает
```bash
# Переустановка Husky
npm run prepare
chmod +x .husky/post-commit
```

### Проблема: Ошибка копирования бинарников
```bash
# Ручная проверка наличия файлов
ls -la target/release/
ls -la vscode-extension/bin/
```

### Проблема: TypeScript не компилируется
```bash
# Проверка зависимостей в расширении
cd vscode-extension
npm install
npm run compile
```

## 📈 Производительность

- **Без автоматизации**: 5-7 минут ручных операций
- **С автоматизацией**: 0 минут (автоматически в фоне)
- **Быстрая разработка**: `npm run rebuild:quick` за 2-3 минуты

## 🔄 Интеграция в CI/CD

Для автоматизации в GitHub Actions добавьте:
```yaml
- name: Build and package extension
  run: npm run rebuild:extension
  
- name: Upload extension artifact
  uses: actions/upload-artifact@v3
  with:
    name: bsl-analyzer-extension
    path: vscode-extension/dist/bsl-analyzer-*.vsix
```