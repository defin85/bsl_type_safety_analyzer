# BSL Type Safety Analyzer - Руководство для LLM-агентов

**Версия:** 1.0  
**Дата:** 2025-07-31  
**Статус:** Черновик

## Обзор

BSL Type Safety Analyzer - это мощный инструмент для валидации кода 1С:Предприятие, оптимизированный для использования LLM-агентами. Проект содержит полную базу знаний о 24,055+ типах BSL с их методами, свойствами и ограничениями.

## 🚀 Быстрый старт для LLM-агентов

### 1. Проверка существования типов и методов

```bash
# Проверить, существует ли метод у типа
cargo run --bin query_type -- --name "Массив" --config "C:\Config\MyConfig" --show-methods | grep "ДобавитьЭлемент"
# Ответ: пусто (метод не существует)

# Найти правильное имя метода
cargo run --bin query_type -- --name "Массив" --config "C:\Config\MyConfig" --show-methods | grep -i "добавить"
# Ответ: Добавить(Значение)

# Проверить конструктор
cargo run --bin query_type -- --name "СписокЗначений" --config "C:\Config\MyConfig"
# Ответ: Тип существует, можно создавать через "Новый СписокЗначений()"
```

### 2. Поиск типов по неточному имени

```bash
# Поиск типа на любом языке
cargo run --bin query_type -- --name "ValueTable" --config "C:\Config\MyConfig" --language auto
# Найдено: ТаблицаЗначений (ValueTable)

# Поиск с учетом режима приложения
cargo run --bin query_type -- --name "УправляемаяФорма" --config "C:\Config\MyConfig" --mode managed
# Найдено: ФормаКлиентскогоПриложения
```

### 3. Проверка совместимости типов

```bash
# Можно ли присвоить один тип другому?
cargo run --bin check_type -- --from "Справочники.Номенклатура" --to "СправочникСсылка" --config "C:\Config\MyConfig"
# Ответ: Compatible

cargo run --bin check_type -- --from "Документы.ЗаказПокупателя" --to "СправочникСсылка" --config "C:\Config\MyConfig"
# Ответ: Not compatible
```

## 📋 Типичные сценарии использования

### Сценарий 1: Валидация сгенерированного кода

**Задача:** LLM сгенерировал код, нужно проверить корректность API

```bsl
// Сгенерированный код
Массив = Новый Массив();
Массив.ДобавитьЭлемент(123);  // Ошибка?
```

**Проверка:**
```bash
# Шаг 1: Проверить методы массива
cargo run --bin query_type -- --name "Массив" --config "путь/к/конфигурации" --show-methods

# Результат покажет правильные методы:
# - Добавить(Значение)
# - Вставить(Индекс, Значение)
# - Удалить(Индекс)
# ...
```

### Сценарий 2: Исправление типичных ошибок LLM

**Частые ошибки и их исправления:**

| Неправильно (LLM) | Правильно | Проверка |
|-------------------|-----------|----------|
| `Новый МассивЗначений()` | `Новый Массив()` | `query_type --name "МассивЗначений"` → Not found |
| `Запрос.ВыполнитьЗапрос()` | `Запрос.Выполнить()` | `query_type --name "Запрос" --show-methods` |
| `Справочник.Название` | `Справочник.Наименование` | `query_type --name "СправочникОбъект" --show-properties` |

### Сценарий 3: Работа с конфигурацией

**Задача:** Найти все справочники с определенным реквизитом

```bash
# Построить индекс конфигурации (с кешированием)
cargo run --bin build_unified_index -- --config "C:\Config\ERP" --platform-version "8.3.25"

# Искать объекты (пока требуется grep, в будущем будет встроенный поиск)
cargo run --bin query_type -- --name "Справочники" --config "C:\Config\ERP" --show-all | grep "ИНН"
```

### Сценарий 4: Проверка платформенной совместимости

```bash
# Проверить доступность метода в разных версиях
cargo run --bin query_type -- --name "HTTPЗапрос" --platform-version "8.3.20" --show-methods
cargo run --bin query_type -- --name "HTTPЗапрос" --platform-version "8.3.25" --show-methods

# Сравнить и найти новые методы
```

## 🤖 Интеграция с AI-агентами

### Claude Code / Cursor / Continue

**Добавить в CLAUDE.md или .cursorrules:**

```markdown
# BSL Validation

При генерации BSL кода ВСЕГДА проверяй:
1. Существование типов: `cargo run --bin query_type -- --name "ИмяТипа" --config "путь"`
2. Методы объектов: добавь флаг `--show-methods`
3. Свойства: добавь флаг `--show-properties`

Типичные ошибки:
- НЕ: Массив.ДобавитьЭлемент() → Массив.Добавить()
- НЕ: Новый СписокЗначений() → Новый СписокЗначений()
- НЕ: Запрос.ВыполнитьЗапрос() → Запрос.Выполнить()
```

### GitHub Copilot

**Создать .github/copilot-instructions.md:**

```markdown
При работе с BSL кодом используй bsl_type_safety_analyzer для валидации:
- Проверяй существование методов перед использованием
- Используй точные имена типов из индекса
- Учитывай режим приложения (ordinary/managed/mixed)
```

### Custom LLM Agent

```python
import subprocess
import json

class BslValidator:
    def __init__(self, config_path):
        self.config_path = config_path
    
    def check_method_exists(self, type_name, method_name):
        """Проверить существование метода у типа"""
        cmd = [
            "cargo", "run", "--bin", "query_type", "--",
            "--name", type_name,
            "--config", self.config_path,
            "--show-methods"
        ]
        result = subprocess.run(cmd, capture_output=True, text=True)
        return method_name in result.stdout
    
    def suggest_correct_method(self, type_name, wrong_method):
        """Предложить правильное имя метода"""
        # Получить все методы
        cmd = [
            "cargo", "run", "--bin", "query_type", "--",
            "--name", type_name,
            "--config", self.config_path,
            "--show-methods"
        ]
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        # Найти похожие (простой пример)
        methods = [line.strip() for line in result.stdout.split('\n') if '(' in line]
        return [m for m in methods if wrong_method.lower() in m.lower()]

# Использование
validator = BslValidator("C:/Config/MyConfig")
if not validator.check_method_exists("Массив", "ДобавитьЭлемент"):
    suggestions = validator.suggest_correct_method("Массив", "Добавить")
    print(f"Метод не найден. Возможно вы имели в виду: {suggestions}")
```

## 📝 Примеры промптов и ожидаемых ответов

### Промпт 1: Создание объекта
```
User: Как создать массив в 1С и добавить в него элемент?
```

**Проверка перед ответом:**
```bash
cargo run --bin query_type -- --name "Массив" --config "./config" --show-methods
```

**Правильный ответ:**
```bsl
Массив = Новый Массив();
Массив.Добавить(123);  // НЕ ДобавитьЭлемент!
```

### Промпт 2: Работа с запросами
```
User: Напиши запрос для получения остатков товаров
```

**Проверка:**
```bash
cargo run --bin query_type -- --name "Запрос" --config "./config" --show-methods
# Убедиться что есть метод Выполнить(), а не ВыполнитьЗапрос()
```

### Промпт 3: Проверка совместимости
```
User: Можно ли передать ссылку на документ в параметр типа СправочникСсылка?
```

**Проверка:**
```bash
cargo run --bin check_type -- --from "ДокументСсылка" --to "СправочникСсылка" --config "./config"
# Ответ: Not compatible
```

## 🔧 Настройка окружения

### Первоначальная настройка

```bash
# 1. Клонировать репозиторий
git clone https://github.com/your/bsl_type_safety_analyzer
cd bsl_type_safety_analyzer

# 2. Установить Rust (если не установлен)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Собрать проект
cargo build --release

# 4. Извлечь платформенные типы (один раз для версии)
cargo run --bin extract_platform_docs -- --archive "path/to/1cv8.zip" --version "8.3.25"

# 5. Построить индекс вашей конфигурации
cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25"
```

### Переменные окружения

```bash
# Указать путь к кешу (опционально)
export BSL_ANALYZER_CACHE_DIR="~/.bsl_analyzer"

# Режим отладки
export RUST_LOG=debug

# Языковые предпочтения по умолчанию
export BSL_DEFAULT_LANGUAGE=russian  # или english, auto
```

## ❓ Troubleshooting и FAQ

### Q: Команда query_type не находит мой тип
**A:** Убедитесь что:
1. Построен индекс конфигурации: `cargo run --bin build_unified_index`
2. Указан правильный путь к конфигурации: `--config "путь"`
3. Тип существует в конфигурации или платформе

### Q: Как обновить кеш после изменения конфигурации?
**A:** Просто запустите `build_unified_index` снова - старый кеш автоматически обновится

### Q: Ошибка "Platform version not found"
**A:** Нужно извлечь документацию для вашей версии:
```bash
cargo run --bin extract_platform_docs -- --archive "1cv8.zip" --version "8.3.XX"
```

### Q: Как проверить код без запуска через cargo?
**A:** После сборки используйте бинарник напрямую:
```bash
./target/release/query_type --name "Массив" --config "./config" --show-methods
```

### Q: Поддерживаются ли английские имена типов?
**A:** Да! Используйте `--language auto` или `--language english`:
```bash
cargo run --bin query_type -- --name "Array" --config "./config" --language auto
# Найдет: Массив (Array)
```

## 🚧 Ограничения текущей версии

1. **BSL Parser не реализован** - нельзя проверить синтаксис целого файла
2. **Нет CLI команды syntaxcheck** - проверка только через отдельные запросы
3. **Нет инкрементального анализа** - только полная переиндексация
4. **Ограниченный поиск** - нужно точное имя типа (нет fuzzy search)

## 📚 Дополнительные ресурсы

- [Архитектура Unified Index](UNIFIED_INDEX_ARCHITECTURE.md)
- [Разработка BSL Grammar](BSL_GRAMMAR_DEVELOPMENT.md)
- [Roadmap проекта](../roadmap.md)
- [CLAUDE.md](../CLAUDE.md) - инструкции для Claude Code

---

*Документ будет обновляться по мере развития проекта и появления новых возможностей.*