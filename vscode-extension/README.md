# BSL Analyzer - VSCode Extension

🚀 **Современное VSCode расширение для анализа BSL (1C:Enterprise) кода с поддержкой унифицированной системы типов**

## 🌟 Ключевые возможности

### 📊 Unified BSL Index Integration
- **Поиск типов**: Мгновенный поиск любого BSL типа (платформенного или конфигурационного)
- **Исследование методов**: Полное отображение методов и свойств типов
- **Проверка совместимости**: Валидация совместимости типов для присваивания
- **Инкрементальные обновления**: Быстрое обновление индекса при изменениях

### 🔍 Language Server Protocol (LSP)
- **Диагностика в реальном времени**: Анализ кода по мере набора
- **Автодополнение**: Контекстные подсказки на основе типов
- **Метрики качества кода**: Визуализация сложности и технического долга
- **Рефакторинг**: Безопасные преобразования кода

### 🎯 Расширенные команды

#### BSL Index
- `BSL Index: Search BSL Type` - Поиск типа по имени
- `BSL Index: Search Method in Type` - Поиск метода в конкретном типе  
- `BSL Index: Build Unified BSL Index` - Построение индекса из конфигурации
- `BSL Index: Show Index Statistics` - Статистика индекса
- `BSL Index: Incremental Index Update` - Инкрементальное обновление
- `BSL Index: Explore Type Methods & Properties` - Детальное исследование типа

#### BSL Verification
- `BSL Verification: Validate Method Call` - Проверка вызова метода
- `BSL Verification: Check Type Compatibility` - Проверка совместимости типов

#### BSL Analyzer
- `BSL Analyzer: Analyze Current File` - Анализ текущего файла
- `BSL Analyzer: Analyze Workspace` - Анализ всего рабочего пространства
- `BSL Analyzer: Generate Reports` - Генерация отчетов
- `BSL Analyzer: Show Code Quality Metrics` - Метрики качества кода

## ⚙️ Настройка

### Обязательные настройки

1. **Путь к бинарным файлам BSL Analyzer**:
   ```json
   "bslAnalyzer.indexServerPath": "C:\\path\\to\\bsl-analyzer\\target\\debug"
   ```

2. **Путь к конфигурации 1С**:
   ```json
   "bslAnalyzer.configurationPath": "C:\\path\\to\\your\\1c\\configuration"
   ```

3. **Версия платформы 1С**:
   ```json
   "bslAnalyzer.platformVersion": "8.3.25"
   ```

### Дополнительные настройки

```json
{
  "bslAnalyzer.autoIndexBuild": true,
  "bslAnalyzer.enableRealTimeAnalysis": true,
  "bslAnalyzer.enableMetrics": true,
  "bslAnalyzer.maxFileSize": 1048576,
  "bslAnalyzer.serverMode": "stdio",
  "bslAnalyzer.trace.server": "off"
}
```

## 🚀 Быстрый старт

### 1. Установка
1. Откройте VSCode
2. Перейдите в Extensions (Ctrl+Shift+X)  
3. Установите "BSL Analyzer"
4. Перезапустите VSCode

### 2. Настройка проекта
1. Откройте настройки (Ctrl+,)
2. Найдите "BSL Analyzer"
3. Укажите пути к бинарным файлам и конфигурации
4. Сохраните настройки

### 3. Построение индекса
1. Откройте Command Palette (Ctrl+Shift+P)
2. Выполните "BSL Index: Build Unified BSL Index"
3. Дождитесь завершения индексации (обычно < 1 секунды)

### 4. Начало работы
- Откройте любой .bsl файл
- Используйте Ctrl+Shift+P для доступа к командам BSL
- Контекстное меню в редакторе для быстрых действий

## 📖 Примеры использования

### Поиск типа
```
Ctrl+Shift+P → "BSL Index: Search BSL Type"
Введите: "Массив" или "Справочники.Номенклатура"
```

### Проверка совместимости типов
```
Ctrl+Shift+P → "BSL Verification: Check Type Compatibility" 
From: "Справочники.Номенклатура"
To: "СправочникСсылка"
→ ✅ СОВМЕСТИМЫ
```

### Валидация вызова метода
1. Выделите вызов метода в коде: `МойОбъект.МойМетод(Параметр1, Параметр2)`
2. Правый клик → "Validate Method Call"
3. Получите детальный анализ совместимости

## 🎯 Архитектура

### UnifiedBslIndex System
- **24,055+ платформенных типов** с автоматическим кешированием
- **Конфигурационные типы** из XML метаданных
- **O(1) поиск** благодаря HashMap индексации  
- **Версионное кеширование** в ~/.bsl_analyzer/

### Performance Optimization (v2.0)
- **Первая индексация**: ~795ms
- **Загрузка из кеша**: ~588ms (25% быстрее)
- **Инкрементальные обновления**: отслеживание изменений файлов
- **Граф наследования**: кеширование связей между типами

## 🔧 Системные требования

- **VSCode**: 1.74.0+
- **1C:Enterprise**: 8.3.24+
- **BSL Analyzer**: v1.2.0+
- **ОС**: Windows 10/11, Linux, macOS

## 🐛 Диагностика проблем

### LSP сервер не запускается
1. Проверьте путь в `bslAnalyzer.serverPath`
2. Убедитесь что lsp_server.exe существует
3. Проверьте права доступа к файлу

### Индекс не строится
1. Проверьте путь к конфигурации в `bslAnalyzer.configurationPath`
2. Убедитесь что у вас есть права на чтение конфигурации
3. Проверьте логи в Output → BSL Analyzer

### Команды не работают
1. Убедитесь что `bslAnalyzer.indexServerPath` указывает на директорию с исполняемыми файлами
2. Проверьте наличие файлов: query_type.exe, build_unified_index.exe, check_type_compatibility.exe
3. Перезапустите VSCode

## 📝 Логи и отладка

Откройте View → Output → BSL Analyzer для просмотра логов:
```
Executing: C:\path\build_unified_index.exe --config "C:\config" --platform-version "8.3.25"
Command completed with code: 0
Output: Index built successfully: 24,055 platform types + 150 config types
```

## 🤝 Поддержка

- **GitHub Issues**: [Сообщить о проблеме](https://github.com/your-org/bsl-analyzer/issues)
- **Documentation**: [docs/](https://github.com/your-org/bsl-analyzer/tree/main/docs)
- **Performance**: [roadmap.md](https://github.com/your-org/bsl-analyzer/blob/main/roadmap.md)

## 📄 Лицензия

MIT License - см. [LICENSE](LICENSE) файл.

---

**🚀 BSL Analyzer v1.2.0** - Unified BSL Type System для современной разработки на 1C:Enterprise