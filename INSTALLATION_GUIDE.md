# Руководство по установке BSL Type Safety Analyzer в VS Code

![BSL Analyzer Logo](vscode-extension/images/bsl-analyzer-logo.png)

## 📦 Установка расширения

### Способ 1: Установка из .vsix файла (рекомендуется)

1. **Соберите .vsix пакет** (если еще не собран):
   ```bash
   cd vscode-extension
   npm install
   npm run compile
   npx @vscode/vsce package
   ```

2. **Установите в VS Code**:
   - Откройте VS Code
   - Нажмите `Ctrl+Shift+P` → введите `Extensions: Install from VSIX`
   - Выберите файл `vscode-extension/bsl-analyzer-1.0.0.vsix`

## 🔧 Настройка для работы с реальным проектом

### 1. Настройте пути к исполняемым файлам

Откройте настройки VS Code (`Ctrl+,`) и найдите секцию `BSL Analyzer`:

```json
{
    "bslAnalyzer.indexServerPath": "C:\\1CProject\\bsl_type_safety_analyzer\\target\\release",
    "bslAnalyzer.serverPath": "C:\\1CProject\\bsl_type_safety_analyzer\\target\\release\\lsp_server.exe",
    "bslAnalyzer.configurationPath": "C:\\Path\\To\\Your\\1C\\Configuration",
    "bslAnalyzer.platformVersion": "8.3.25"
}
```

### 2. Подготовьте платформенную документацию (один раз)

```bash
# Извлеките документацию 1С платформы
cargo run --bin extract_platform_docs -- --archive "examples/rebuilt.shcntx_ru.zip" --platform-version "8.3.25"
```

### 3. Постройте индекс вашей конфигурации

```bash
# Замените путь на путь к вашей конфигурации
cargo run --bin build_unified_index -- --config "C:\\Path\\To\\Your\\1C\\Configuration" --platform-version "8.3.25"
```

## 🚀 Основные возможности

### Команды доступные в VS Code:

1. **`Ctrl+Shift+P` → "BSL Analyzer: Build Unified BSL Index"**  
   Построить индекс типов для текущего проекта

2. **`Ctrl+Shift+P` → "BSL Analyzer: Search BSL Type"**  
   Поиск типов в индексе (например, "Справочники.Номенклатура")

3. **`Ctrl+Shift+P` → "BSL Analyzer: Analyze Current File"**  
   Анализ текущего BSL файла

4. **`Ctrl+Shift+P` → "BSL Analyzer: Show Code Quality Metrics"**  
   Показать метрики качества кода

### Контекстное меню:

- **Правый клик на .bsl файле** → доступны команды анализа
- **Выделите код** → доступна валидация вызовов методов

## 🔧 Возможные настройки

```json
{
    "bslAnalyzer.enabled": true,
    "bslAnalyzer.autoIndexBuild": false,
    "bslAnalyzer.enableRealTimeAnalysis": true,
    "bslAnalyzer.enableMethodValidation": true,
    "bslAnalyzer.maxFileSize": 1048576,
    "bslAnalyzer.trace.server": "off"
}
```

## 🎯 Тестирование на реальном проекте

1. **Откройте папку с BSL файлами** в VS Code
2. **Настройте путь к конфигурации** в настройках расширения
3. **Выполните команду** "Build Unified BSL Index"
4. **Откройте любой .bsl файл** и увидите:
   - Подсветку синтаксиса
   - Автодополнение методов
   - Диагностические сообщения
   - Валидацию типов

## 📝 Поддерживаемые файлы

- `.bsl` - основные модули BSL
- `.os` - модули OneScript

## 🐛 Отладка

При проблемах включите подробное логирование:

```json
{
    "bslAnalyzer.trace.server": "verbose"
}
```

И проверьте вывод в панели "Output" → "BSL Analyzer".

## 📚 Дополнительные возможности

- **Поиск по индексу типов**: 24,000+ платформенных типов + ваши типы конфигурации
- **Валидация методов**: проверка существования и сигнатур методов  
- **Метрики качества**: цикломатическая сложность, использование переменных
- **Автодополнение**: методы и свойства всех BSL типов