# 🧪 Инструкции по тестированию системы прогресса

## ✅ Успешно работает:
- Тестовая команда прогресса запускается вручную и показывает корректную индикацию

## 🚀 Тестирование реальных команд индексации:

### 1. Настройка конфигурации

Откройте настройки VSCode (`Ctrl+,`) и найдите "BSL Analyzer":

```json
{
    "bslAnalyzer.configurationPath": "C:\\1CProject\\bsl_type_safety_analyzer\\examples\\ConfTest",
    "bslAnalyzer.platformVersion": "8.3.25"
}
```

### 2. Команды для тестирования прогресса:

#### A) Тест системы прогресса (работает ✅)
- `Ctrl+Shift+P` → `"BSL Debug: Test Progress System"`
- **Ожидаемый результат**: 5 этапов по 2 секунды с прогрессом в статус-баре и панели

#### B) Построение индекса BSL
- `Ctrl+Shift+P` → `"BSL Index: Build Unified BSL Index"`  
- **Ожидаемый результат**: 4 этапа с прогрессом:
  1. Loading platform cache... (10%)
  2. Parsing configuration... (35%) 
  3. Building unified index... (70%)
  4. Finalizing index... (90%)

#### C) Инкрементальное обновление
- `Ctrl+Shift+P` → `"BSL Index: Incremental Index Update"`
- **Ожидаемый результат**: 3 этапа с прогрессом

#### D) Добавление платформенной документации
- Перейти в панель "BSL Analyzer" в Activity Bar
- Найти секцию "Platform Documentation" 
- Нажать "➕ Add Platform Documentation..."
- **Ожидаемый результат**: 3 этапа парсинга с прогрессом

### 3. Места для наблюдения прогресса:

#### 🎯 Статусная строка (внизу VSCode)
```
$(loading~spin) BSL: Loading platform cache... (10%)
```

#### 🎯 Панель Overview (боковая панель)
- Динамический элемент "$(loading~spin) Loading platform cache... (10%)"
- Tooltip с деталями: ETA, шаг N/M

#### 🎯 VSCode Notification
- Прогресс-бар с описанием этапов

#### 🎯 Output Channel
- Подробные логи всех операций
- View → Output → "BSL Analyzer"

### 4. Проверка корректности:

✅ **Правильная работа**:
- Спиннер появляется в статус-баре при старте
- Проценты увеличиваются по мере выполнения
- Панель Overview обновляется автоматически
- ETA рассчитывается для долгих операций
- После завершения статус меняется на "BSL Analyzer: Index Ready"

❌ **Возможные проблемы**:
- Статус-бар не меняется (проблема с updateStatusBar)
- Прогресс сразу исчезает (слишком быстрое выполнение)
- Панель не обновляется (проблема с event emitter)

### 5. Отладочная информация:

Если прогресс не отображается:
1. Откройте Output Channel: View → Output → "BSL Analyzer"
2. Ищите логи типа:
```
🚀 Index building started with 4 steps
📊 Step 1/4: Loading platform cache... (10%)
📊 StatusBar text: $(loading~spin) BSL: Loading platform cache... (10%)
```

3. Проверьте наличие ошибок:
```
⚠️ updateIndexingProgress called but indexing is not active
❌ LSP client startup failed: ...
```

## 🎯 Результаты тестирования

После тестирования обновите этот файл результатами:

- [ ] Test Progress System: ✅ Работает
- [ ] Build Unified BSL Index: ___ (результат)
- [ ] Incremental Index Update: ___ (результат)  
- [ ] Add Platform Documentation: ___ (результат)

## 📝 Примечания

- Тестовая конфигурация `examples/ConfTest` содержит 5 объектов
- Время выполнения команд может быть быстрым (< 1 секунды)
- Для демонстрации прогресса добавлены искусственные задержки в тестовой команде