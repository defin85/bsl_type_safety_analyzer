# Унификация логики анализатора BSL

## 📊 Текущая ситуация

### Существующие анализаторы

1. **BslAnalyzer** (`src/bsl_parser/analyzer.rs`)
   - Главный унифицированный анализатор
   - Объединяет: BslParser, SemanticAnalyzer, DataFlowAnalyzer
   - Поддерживает UnifiedBslIndex
   - Используется в MCP сервере

2. **BslParser** (`src/bsl_parser/mod.rs`)
   - Низкоуровневый tree-sitter парсер
   - Используется напрямую в `syntaxcheck.rs`
   - Базовый синтаксический анализ

3. **SemanticAnalyzer** (`src/bsl_parser/semantic.rs`)
   - Семантический анализ
   - Проверка типов и методов
   - Интегрирован в BslAnalyzer

4. **DataFlowAnalyzer** (`src/bsl_parser/data_flow.rs`)
   - Анализ потока данных
   - Проверка инициализации переменных
   - Интегрирован в BslAnalyzer

### 🔍 Проблемы дублирования

#### 1. Прямое использование BslParser
**Файл:** `src/bin/syntaxcheck.rs`
```rust
let mut parser = BslParser::new()?;
// Напрямую использует парсер без семантического анализа
```
**Проблема:** Не использует возможности BslAnalyzer

#### 2. LSP сервер не использует BslAnalyzer
**Файл:** `src/lsp/server.rs`
- Работает с UnifiedBslIndex напрямую
- Не использует готовый BslAnalyzer для диагностики
- Дублирует логику анализа

#### 3. Несогласованность уровней анализа
- CLI утилиты используют разные уровни анализа
- Нет единой точки входа для всех видов анализа

## 🎯 План унификации

### Фаза 1: Расширение BslAnalyzer

#### 1.1 Добавить уровни анализа
```rust
pub enum AnalysisLevel {
    Syntax,           // Только синтаксис
    Semantic,         // + семантический анализ
    DataFlow,         // + анализ потока данных
    Full,            // Полный анализ
}
```

#### 1.2 Добавить конфигурацию анализа
```rust
pub struct AnalysisConfig {
    pub level: AnalysisLevel,
    pub check_method_calls: bool,
    pub check_type_compatibility: bool,
    pub check_unused_variables: bool,
    pub check_uninitialized: bool,
}
```

#### 1.3 Унифицированный API
```rust
impl BslAnalyzer {
    pub fn analyze_file(&mut self, path: &Path, config: &AnalysisConfig) -> AnalysisResult;
    pub fn analyze_text(&mut self, text: &str, config: &AnalysisConfig) -> AnalysisResult;
    pub fn analyze_module(&mut self, module: &Module, config: &AnalysisConfig) -> AnalysisResult;
}
```

### Фаза 2: Рефакторинг CLI утилит

#### 2.1 syntaxcheck.rs
```rust
// Было:
let mut parser = BslParser::new()?;
let result = parser.parse(&content, path.to_str().unwrap_or("<unknown>"));

// Стало:
let mut analyzer = BslAnalyzer::new()?;
let config = AnalysisConfig::syntax_only();
let result = analyzer.analyze_text(&content, &config)?;
```

#### 2.2 Создать главную CLI утилиту
`src/bin/bsl-analyzer.rs` - единая точка входа для анализа

### Фаза 3: Интеграция с LSP

#### 3.1 Использовать BslAnalyzer в LSP
```rust
struct BslLanguageServer {
    analyzer: Arc<RwLock<BslAnalyzer>>,
    // ...
}
```

#### 3.2 Единая диагностика
- Использовать одинаковые Diagnostic структуры
- Общий формат ошибок и предупреждений

### Фаза 4: Унификация результатов

#### 4.1 Единая структура AnalysisResult
```rust
pub struct AnalysisResult {
    pub diagnostics: Vec<Diagnostic>,
    pub metrics: AnalysisMetrics,
    pub ast: Option<AstNode>,
    pub semantic_info: Option<SemanticInfo>,
}
```

#### 4.2 Конвертеры для разных форматов
- LSP диагностика
- CLI вывод (text, json, sarif)
- MCP формат

## 📋 Детальный план реализации

### ✅ Шаг 1: Расширить BslAnalyzer (ЗАВЕРШЕНО)
- [x] Добавить AnalysisLevel enum (Syntax, Semantic, DataFlow, Full)
- [x] Создать AnalysisConfig struct с настройками анализа
- [x] Реализовать analyze_file/analyze_text методы
- [x] Добавить поддержку разных уровней анализа

### ✅ Шаг 2: Рефакторить syntaxcheck.rs (ЗАВЕРШЕНО)
- [x] Заменить BslParser на BslAnalyzer
- [x] Использовать AnalysisConfig
- [x] Сохранить совместимость вывода

### ✅ Шаг 3: Создать главную CLI утилиту (ЗАВЕРШЕНО)
- [x] Создать src/bin/bsl-analyzer.rs с 7 подкомандами
- [x] Добавить подкоманды: analyze, check, validate, index, find, compat, stats
- [x] Интегрировать с cli_common
- [x] Поддержка всех уровней анализа и форматов вывода

### Шаг 4: Интегрировать с LSP (3 часа)
- [ ] Добавить BslAnalyzer в BslLanguageServer
- [ ] Использовать единую диагностику
- [ ] Обновить обработчики документов

### Шаг 5: Тестирование и документация (2 часа)
- [ ] Написать тесты для разных уровней анализа
- [ ] Обновить документацию
- [ ] Проверить производительность

## 🎯 Ожидаемые результаты

### Преимущества унификации

1. **Единая логика анализа**
   - Одинаковые результаты в CLI и LSP
   - Консистентная диагностика

2. **Легкость расширения**
   - Новые проверки добавляются в одном месте
   - Автоматически доступны везде

3. **Улучшенная производительность**
   - Кеширование результатов анализа
   - Инкрементальный анализ

4. **Упрощенная поддержка**
   - Меньше дублирования кода
   - Единая точка для исправления багов

### Метрики успеха

- Устранено 500+ строк дублированного кода
- Единый API для всех видов анализа
- 100% покрытие тестами основных сценариев
- Одинаковые результаты в CLI/LSP/MCP

## 🚀 Приоритеты реализации

1. **Высокий приоритет**
   - Расширение BslAnalyzer
   - Рефакторинг syntaxcheck.rs

2. **Средний приоритет**
   - Создание главной CLI утилиты
   - Интеграция с LSP

3. **Низкий приоритет**
   - Оптимизация производительности
   - Расширенная документация