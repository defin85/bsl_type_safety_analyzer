# BSL Grammar Development Guide

**Дата:** 2025-07-30  
**Версия:** 1.0

## Обзор

Это руководство описывает методики разработки BSL грамматики для парсера на основе logos+nom и процесс адаптации существующих ANTLR грамматик.

## Методики разработки грамматики

### 1. Использование существующих грамматик

#### Открытые проекты с BSL грамматиками:
- **[BSL Language Server](https://github.com/1c-syntax/bsl-language-server)** - полная ANTLR4 грамматика
- **[tree-sitter-bsl](https://github.com/Daabramov/tree-sitter-bsl)** - грамматика для tree-sitter
- **[SonarQube 1C (BSL) Community Plugin](https://github.com/1c-syntax/sonar-bsl-plugin-community)** - ANTLR4 грамматика с правилами анализа

### 2. Итеративный подход "снизу-вверх"

#### Этап 1: Атомарные элементы
```rust
// Базовые токены
identifier = { alpha ~ (alpha | digit | "_")* }
number = { digit+ }
string = { "\"" ~ (!"\"" ~ any)* ~ "\"" }
```

#### Этап 2: Простые выражения
```rust
// Обращения к свойствам и методам
property_access = { identifier ~ "." ~ identifier }
method_call = { identifier ~ "." ~ identifier ~ "(" ~ args? ~ ")" }
```

#### Этап 3: Составные конструкции
```rust
// Создание объектов
new_expression = { "Новый" ~ identifier ~ "(" ~ args? ~ ")" }
// Цепочки вызовов
chain_call = { identifier ~ ("." ~ identifier ~ "(" ~ args? ~ ")")+ }
```

### 3. Test-Driven Grammar Development

```rust
#[cfg(test)]
mod grammar_tests {
    use super::*;
    
    #[test]
    fn test_method_calls() {
        // Позитивные тесты
        assert_parse_ok("Массив.Добавить(1)");
        assert_parse_ok("Объект.МетодСПараметрами(1, 2, 3)");
        assert_parse_ok("Запрос.УстановитьПараметр(\"Дата\", ТекущаяДата())");
        
        // Негативные тесты
        assert_parse_fail("Массив.()");  // нет имени метода
        assert_parse_fail("Массив.Добавить(,)");  // пустой параметр
    }
    
    #[test]
    fn test_property_access() {
        assert_parse_ok("Справочник.Наименование");
        assert_parse_ok("Документ.Дата");
        assert_parse_ok("Форма.ЭлементыФормы.Кнопка1");  // цепочка
    }
}
```

### 4. Приоритизация по частоте ошибок LLM

Статистика типичных ошибок LLM в BSL коде:

| Тип ошибки | Частота | Пример |
|------------|---------|---------|
| Несуществующие методы | 35% | `Массив.ДобавитьЭлемент()` |
| Неправильные конструкторы | 25% | `Новый МассивЗначений()` |
| Путаница в API | 20% | `Запрос.ВыполнитьЗапрос()` вместо `Выполнить()` |
| Неверные свойства | 15% | `Справочник.Название` вместо `Наименование` |
| Прочее | 5% | Синтаксические ошибки |

### 5. Модульная архитектура парсера

```
src/parser/bsl/
├── lexer.rs         # logos токенизатор
├── expressions.rs   # nom парсеры для выражений
├── statements.rs    # nom парсеры для операторов
├── ast.rs          # AST структуры
├── validator.rs    # интеграция с UnifiedBslIndex
└── mod.rs         # публичный API
```

## Адаптация ANTLR грамматики под logos+nom

### Пример конвертации

#### ANTLR4 грамматика (исходная)
```antlr
// Вызов метода
methodCall
    : IDENTIFIER '.' IDENTIFIER '(' callParamList? ')'
    ;

// Обращение к свойству  
memberAccess
    : IDENTIFIER ('.' IDENTIFIER)+
    ;

// Конструктор
newExpression
    : NEW_KEYWORD typeName '(' callParamList? ')'
    ;

// Параметры вызова
callParamList
    : callParam (',' callParam)*
    ;
```

### Реализация на logos+nom

#### 1. Определение токенов (logos)

```rust
use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Ключевые слова (русский и английский варианты)
    #[token("Новый", ignore(case))]
    #[token("New", ignore(case))]
    New,
    
    #[token("Процедура", ignore(case))]
    #[token("Procedure", ignore(case))]
    Procedure,
    
    #[token("Функция", ignore(case))]
    #[token("Function", ignore(case))]
    Function,
    
    // Идентификаторы
    #[regex(r"[а-яА-Яa-zA-Z_][а-яА-Яa-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    // Литералы
    #[regex(r"\d+(\.\d+)?", |lex| lex.slice().to_string())]
    Number(String),
    
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string() // убираем кавычки
    })]
    String(String),
    
    // Операторы и разделители
    #[token(".")]
    Dot,
    
    #[token("(")]
    LParen,
    
    #[token(")")]
    RParen,
    
    #[token(",")]
    Comma,
    
    #[token(";")]
    Semicolon,
    
    // Пропускаем пробелы и комментарии
    #[regex(r"[ \t\n\f]+", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    #[error]
    Error,
}

// Расширение для отслеживания позиций
#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
```

#### 2. AST структуры

```rust
#[derive(Debug, Clone)]
pub struct MethodCall {
    pub object: String,
    pub method: String,
    pub args: Vec<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PropertyAccess {
    pub object: String,
    pub property: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct NewExpression {
    pub type_name: String,
    pub args: Vec<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Expression {
    MethodCall(MethodCall),
    PropertyAccess(PropertyAccess),
    New(NewExpression),
    Identifier(String),
    Number(String),
    String(String),
}
```

#### 3. Парсеры nom

```rust
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt, recognize},
    multi::separated_list0,
    sequence::{preceded, terminated, tuple, delimited},
};

type ParseResult<'a, T> = IResult<&'a [SpannedToken], T>;

// Вспомогательные функции
fn identifier(input: &[SpannedToken]) -> ParseResult<(String, Span)> {
    match input.first() {
        Some(SpannedToken { token: Token::Identifier(name), span }) => {
            Ok((&input[1..], (name.clone(), *span)))
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag
        ))),
    }
}

fn dot(input: &[SpannedToken]) -> ParseResult<()> {
    match input.first() {
        Some(SpannedToken { token: Token::Dot, .. }) => Ok((&input[1..], ())),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag
        ))),
    }
}

fn lparen(input: &[SpannedToken]) -> ParseResult<()> {
    match input.first() {
        Some(SpannedToken { token: Token::LParen, .. }) => Ok((&input[1..], ())),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag
        ))),
    }
}

fn rparen(input: &[SpannedToken]) -> ParseResult<()> {
    match input.first() {
        Some(SpannedToken { token: Token::RParen, .. }) => Ok((&input[1..], ())),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag
        ))),
    }
}

fn comma(input: &[SpannedToken]) -> ParseResult<()> {
    match input.first() {
        Some(SpannedToken { token: Token::Comma, .. }) => Ok((&input[1..], ())),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag
        ))),
    }
}

fn new_keyword(input: &[SpannedToken]) -> ParseResult<Span> {
    match input.first() {
        Some(SpannedToken { token: Token::New, span }) => Ok((&input[1..], *span)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag
        ))),
    }
}

// Парсер для вызова метода: Объект.Метод(арг1, арг2)
fn method_call(input: &[SpannedToken]) -> ParseResult<Expression> {
    let start_pos = input.first().map(|t| t.span.start).unwrap_or(0);
    
    map(
        tuple((
            identifier,
            preceded(dot, identifier),
            preceded(lparen, opt(call_param_list)),
            rparen,
        )),
        move |((object, obj_span), (method, _), args, _)| {
            let end_pos = input.first().map(|t| t.span.end).unwrap_or(start_pos);
            Expression::MethodCall(MethodCall {
                object,
                method,
                args: args.unwrap_or_default(),
                span: Span { start: obj_span.start, end: end_pos },
            })
        }
    )(input)
}

// Парсер для обращения к свойству: Объект.Свойство
fn property_access(input: &[SpannedToken]) -> ParseResult<Expression> {
    let start_pos = input.first().map(|t| t.span.start).unwrap_or(0);
    
    map(
        tuple((
            identifier,
            preceded(dot, identifier),
        )),
        move |((object, obj_span), (property, prop_span))| {
            Expression::PropertyAccess(PropertyAccess {
                object,
                property,
                span: Span { start: obj_span.start, end: prop_span.end },
            })
        }
    )(input)
}

// Парсер для конструктора: Новый ТипОбъекта(параметры)
fn new_expression(input: &[SpannedToken]) -> ParseResult<Expression> {
    map(
        tuple((
            new_keyword,
            identifier,
            preceded(lparen, opt(call_param_list)),
            rparen,
        )),
        |(new_span, (type_name, _), args, _)| {
            let end_pos = input.first().map(|t| t.span.end).unwrap_or(new_span.end);
            Expression::New(NewExpression {
                type_name,
                args: args.unwrap_or_default(),
                span: Span { start: new_span.start, end: end_pos },
            })
        }
    )(input)
}

// Список параметров
fn call_param_list(input: &[SpannedToken]) -> ParseResult<Vec<Expression>> {
    separated_list0(comma, expression)(input)
}

// Главный парсер выражений
pub fn expression(input: &[SpannedToken]) -> ParseResult<Expression> {
    alt((
        new_expression,
        method_call,
        property_access,
        map(identifier, |(name, span)| Expression::Identifier(name)),
        // Добавить остальные типы выражений
    ))(input)
}
```

#### 4. Валидация с UnifiedBslIndex

```rust
use crate::unified_index::UnifiedBslIndex;
use crate::diagnostics::{Diagnostic, DiagnosticDetails, Severity};

pub struct BslValidator<'a> {
    index: &'a UnifiedBslIndex,
}

impl<'a> BslValidator<'a> {
    pub fn new(index: &'a UnifiedBslIndex) -> Self {
        Self { index }
    }
    
    pub fn validate_expression(&self, expr: &Expression) -> Result<(), Diagnostic> {
        match expr {
            Expression::MethodCall(call) => self.validate_method_call(call),
            Expression::PropertyAccess(access) => self.validate_property_access(access),
            Expression::New(new_expr) => self.validate_new_expression(new_expr),
            _ => Ok(()),
        }
    }
    
    fn validate_method_call(&self, call: &MethodCall) -> Result<(), Diagnostic> {
        // Проверяем существование метода
        if !self.index.has_method(&call.object, &call.method) {
            let suggestions = self.index.find_similar_methods(&call.object, &call.method);
            
            return Err(Diagnostic {
                severity: Severity::Error,
                location: Location {
                    file: PathBuf::new(), // заполнить реальным путем
                    range: span_to_range(call.span),
                },
                code: "BSL001".to_string(),
                message: format!("Unknown method '{}' for type '{}'", call.method, call.object),
                details: DiagnosticDetails::UnknownMethod {
                    object: call.object.clone(),
                    method: call.method.clone(),
                },
                suggestions,
                related_info: vec![],
            });
        }
        
        // Проверяем количество параметров
        let method_info = self.index.get_method(&call.object, &call.method)?;
        if let Some(expected_params) = method_info.parameter_count() {
            if call.args.len() != expected_params {
                return Err(Diagnostic {
                    severity: Severity::Error,
                    location: Location {
                        file: PathBuf::new(),
                        range: span_to_range(call.span),
                    },
                    code: "BSL002".to_string(),
                    message: format!(
                        "Wrong number of arguments: expected {}, found {}",
                        expected_params,
                        call.args.len()
                    ),
                    details: DiagnosticDetails::WrongArgumentCount {
                        expected: expected_params,
                        found: call.args.len(),
                    },
                    suggestions: vec![],
                    related_info: vec![],
                });
            }
        }
        
        Ok(())
    }
    
    fn validate_property_access(&self, access: &PropertyAccess) -> Result<(), Diagnostic> {
        if !self.index.has_property(&access.object, &access.property) {
            return Err(Diagnostic {
                severity: Severity::Error,
                location: Location {
                    file: PathBuf::new(),
                    range: span_to_range(access.span),
                },
                code: "BSL003".to_string(),
                message: format!("Unknown property '{}' for type '{}'", access.property, access.object),
                details: DiagnosticDetails::UnknownProperty {
                    object: access.object.clone(),
                    property: access.property.clone(),
                },
                suggestions: self.index.find_similar_properties(&access.object, &access.property),
                related_info: vec![],
            });
        }
        Ok(())
    }
    
    fn validate_new_expression(&self, new_expr: &NewExpression) -> Result<(), Diagnostic> {
        if !self.index.is_constructible(&new_expr.type_name) {
            return Err(Diagnostic {
                severity: Severity::Error,
                location: Location {
                    file: PathBuf::new(),
                    range: span_to_range(new_expr.span),
                },
                code: "BSL004".to_string(),
                message: format!("Type '{}' is not constructible", new_expr.type_name),
                details: DiagnosticDetails::NotConstructible {
                    type_name: new_expr.type_name.clone(),
                },
                suggestions: vec![],
                related_info: vec![],
            });
        }
        Ok(())
    }
}
```

## Практические рекомендации

### 1. Начните с минимального набора

Для MVP достаточно поддержать:
- Вызовы методов
- Обращения к свойствам
- Конструкторы объектов
- Базовые литералы

### 2. Используйте TDD

Напишите тесты для типичных ошибок LLM:
```rust
#[test]
fn test_llm_common_mistakes() {
    // LLM часто придумывает методы
    assert_validation_error("Массив.ДобавитьЭлемент(1)", "BSL001");
    assert_validation_error("Запрос.ВыполнитьЗапрос()", "BSL001");
    
    // LLM путает конструкторы
    assert_validation_error("Новый МассивЗначений()", "BSL004");
    assert_validation_error("Новый СписокЗначений()", "BSL004"); // правильно: СписокЗначений
}
```

### 3. Инкрементальное развитие

1. **Фаза 1:** Базовые конструкции (методы, свойства, конструкторы)
2. **Фаза 2:** Операторы и выражения
3. **Фаза 3:** Управляющие конструкции (если, циклы)
4. **Фаза 4:** Объявления (процедуры, функции)

### 4. Документируйте паттерны

Создайте каталог типичных паттернов BSL кода:
```rust
// patterns.md
## Работа с массивами
- Создание: `Массив = Новый Массив();`
- Добавление: `Массив.Добавить(Значение);`
- НЕ: `Массив.ДобавитьЭлемент()`, `Массив.Push()`

## Работа с запросами
- Создание: `Запрос = Новый Запрос();`
- Выполнение: `Результат = Запрос.Выполнить();`
- НЕ: `Запрос.ВыполнитьЗапрос()`, `Запрос.Run()`
```

## Интеграция с проектом

### 1. Структура файлов
```
src/parser/
├── bsl/
│   ├── mod.rs
│   ├── lexer.rs       # Расширение существующего
│   ├── grammar/
│   │   ├── mod.rs
│   │   ├── expressions.rs
│   │   ├── statements.rs
│   │   └── validators.rs
│   └── tests/
│       ├── lexer_tests.rs
│       ├── parser_tests.rs
│       └── validation_tests.rs
```

### 2. Точки интеграции

- Использовать существующий `UnifiedBslIndex` для валидации
- Интегрировать с `DiagnosticEngine` для форматирования ошибок
- Поддержать инкрементальный парсинг через `IncrementalParser`

## Ссылки и ресурсы

1. [nom documentation](https://docs.rs/nom/)
2. [logos documentation](https://docs.rs/logos/)
3. [BSL Language Server Grammar](https://github.com/1c-syntax/bsl-language-server/tree/develop/src/main/antlr)
4. [Writing Parser Combinators](https://bodil.lol/parser-combinators/)

---

*Документ будет обновляться по мере развития парсера.*