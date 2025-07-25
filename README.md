# BSL Type Safety Analyzer

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tests](https://github.com/your-username/bsl-type-safety-analyzer/workflows/Tests/badge.svg)](https://github.com/your-username/bsl-type-safety-analyzer/actions)

Статический анализатор типобезопасности для языка 1С BSL, написанный на Rust. Предоставляет комплексный анализ кода BSL для выявления проблем типобезопасности, качества кода и соответствия лучшим практикам.

## 🚀 Быстрый старт

### Установка

```bash
# Скачать и собрать из исходников
git clone https://github.com/your-username/bsl-type-safety-analyzer.git
cd bsl-type-safety-analyzer
cargo build --release

# Запуск
./target/release/bsl-analyzer --help
```

### Использование

```bash
# Анализ файла BSL
bsl-analyzer analyze file.bsl

# Анализ конфигурации
bsl-analyzer analyze --config config.xml

# Запуск LSP сервера для VS Code
bsl-analyzer lsp --port 3000

# Валидация конфигурации
bsl-analyzer validate config.xml

# Получение информации
bsl-analyzer info --config config.xml
```

## 📋 Возможности

### 🔍 Многоуровневый анализ
- **Лексический анализ**: Токенизация и базовая проверка синтаксиса
- **Синтаксический анализ**: Проверка структуры кода и грамматики BSL
- **Семантический анализ**: Анализ типов, областей видимости и логики
- **Анализ потока данных**: Отслеживание переменных и их значений
- **Верификация методов**: Проверка существования и корректности вызовов методов

### 🛡️ Типобезопасность
- Проверка типов переменных и параметров
- Валидация вызовов методов и функций
- Контроль совместимости типов
- Анализ приведений типов

### 📊 Качество кода
- Выявление неиспользуемых переменных
- Проверка инициализации переменных
- Анализ сложности кода
- Рекомендации по оптимизации

### ⚡ Производительность
- Кэширование результатов анализа
- Многопоточная обработка больших проектов
- Оптимизированные алгоритмы анализа
- Метрики производительности

## 🏗️ Архитектура

```
bsl_type_safety_analyzer/
├── core/                    # Основные компоненты
│   ├── analyzer.py         # Главный анализатор
│   ├── base_analyzer.py    # Базовый класс для анализаторов
│   ├── config.py           # Конфигурация
│   ├── errors.py           # Система ошибок
│   ├── logger.py           # Логирование
│   ├── validators.py       # Валидация
│   ├── cache.py            # Кэширование
│   └── metrics.py          # Метрики
├── analyzers/              # Модули анализа
│   ├── lexical_analyzer.py
│   ├── syntax_analyzer.py
│   ├── semantic_analyzer.py
│   └── data_flow_analyzer.py
├── verifiers/              # Верификаторы
│   └── method_verifier.py
├── utils/                  # Утилиты
│   ├── code_formatter.py
│   └── documentation_integrator.py
├── docs_search/            # Документация 1С
└── examples/               # Примеры использования
```

## 📖 Документация

### Конфигурация

```python
from bsl_type_safety_analyzer import AnalyzerConfig, ErrorLevel

config = AnalyzerConfig(
    error_level=ErrorLevel.WARNING,
    enable_cache=True,
    cache_ttl=3600,
    max_file_size=1024*1024,
    include_patterns=["*.bsl", "*.os"],
    exclude_patterns=["*/vendor/*", "*/tests/*"]
)
```

### API

#### Основные функции

```python
# Анализ кода
result = analyze_code(code: str, config: AnalyzerConfig = None) -> AnalysisResult

# Анализ файла
result = analyze_file(file_path: str, config: AnalyzerConfig = None) -> AnalysisResult

# Анализ проекта
result = analyze_project(project_path: str, config: AnalyzerConfig = None) -> AnalysisResult
```

#### Класс анализатора

```python
from bsl_type_safety_analyzer import BSLTypeSafetyAnalyzer

analyzer = BSLTypeSafetyAnalyzer(config)

# Анализ с контекстом
result = analyzer.analyze_code(code, context=AnalysisContext(
    file_path="example.bsl",
    project_root="/path/to/project"
))
```

### CLI команды

```bash
# Анализ файла
bsl-analyzer analyze-file path/to/file.bsl [--output json] [--verbose]

# Анализ проекта
bsl-analyzer analyze-project path/to/project [--recursive] [--output html]

# Показать справку
bsl-analyzer --help
bsl-analyzer analyze-file --help
```

## 🧪 Тестирование

```bash
# Запуск всех тестов
python -m pytest tests/ -v

# Запуск с покрытием
python -m pytest tests/ --cov=bsl_type_safety_analyzer --cov-report=html

# Запуск конкретного теста
python -m pytest tests/test_lexical_analyzer.py::test_tokenize_basic -v
```

## 🚀 Разработка

### Установка для разработки

```bash
# Клонирование репозитория
git clone https://github.com/your-username/bsl-type-safety-analyzer.git
cd bsl-type-safety-analyzer

# Установка зависимостей
pip install -r requirements.txt
pip install -e .

# Установка dev зависимостей
pip install -e ".[dev]"
```

### Сборка и публикация

```bash
# Сборка пакета
python build_package.py

# Или вручную
python -m build
twine check dist/*
twine upload dist/*
```

## 📊 Метрики и мониторинг

Анализатор предоставляет детальную статистику:

- Время выполнения анализа
- Количество обработанных файлов
- Статистика ошибок по типам
- Использование памяти
- Производительность кэша

## 🤝 Вклад в проект

1. Форкните репозиторий
2. Создайте ветку для новой функции (`git checkout -b feature/amazing-feature`)
3. Зафиксируйте изменения (`git commit -m 'Add amazing feature'`)
4. Отправьте в ветку (`git push origin feature/amazing-feature`)
5. Откройте Pull Request

## 📄 Лицензия

Этот проект лицензирован под MIT License - см. файл [LICENSE](LICENSE) для деталей.

## 🙏 Благодарности

- Команда 1С за документацию платформы
- Сообщество разработчиков 1С за обратную связь
- Авторы используемых библиотек

## 📞 Поддержка

- 📧 Email: bsl-analyzer@example.com
- 🐛 Issues: [GitHub Issues](https://github.com/your-username/bsl-type-safety-analyzer/issues)
- 📖 Документация: [Wiki](https://github.com/your-username/bsl-type-safety-analyzer/wiki)

---

**BSL Type Safety Analyzer** - делаем код 1С безопаснее и качественнее! 🛡️ 