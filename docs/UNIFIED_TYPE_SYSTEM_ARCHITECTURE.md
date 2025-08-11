# Архитектура единой системы типов BSL

## Обзор

Система типов 1С:Предприятие имеет уникальную структуру, которая не укладывается в классические парадигмы ООП. Данный документ описывает архитектуру "Unified Type System with Facets" - подход к представлению типов 1С в анализаторе BSL.

## Ключевые концепции

### 1. Проблема множественных представлений

Каждая сущность конфигурации 1С может иметь несколько представлений:

```bsl
// Runtime представления - работа с данными
Справочники.Контрагенты                    // СправочникМенеджер.Контрагенты
Справочники.Контрагенты.СоздатьЭлемент()   // СправочникОбъект.Контрагенты
Справочники.Контрагенты.НайтиПоКоду("123") // СправочникСсылка.Контрагенты

// Metadata представления - работа со структурой
Метаданные.Справочники.Контрагенты         // ОбъектМетаданныхСправочник
Метаданные.Справочники.Контрагенты.Реквизиты // КоллекцияОбъектовМетаданных
```

### 2. Решение: Фасеты (Facets)

Вместо создания отдельных типов для каждого представления, используем единый тип с различными "фасетами" - гранями, которые активируются в зависимости от контекста использования.

## Архитектура системы

### Трёхуровневая структура хранения

```rust
pub struct UnifiedBslIndex {
    // Уровень 1: Шаблоны фасетов (из платформы)
    facet_templates: FacetTemplateRegistry,
    
    // Уровень 2: Конфигурационные типы
    configuration_types: HashMap<String, UnifiedBslType>,
    
    // Уровень 3: Глобальный контекст
    global_context: GlobalContext,
}
```

### Уровень 1: Шаблоны фасетов

Хранятся ОДИН раз для всех типов одной категории. Содержат методы и свойства из платформы.

```rust
pub struct FacetTemplateRegistry {
    catalog_facets: CatalogFacets {
        manager: ManagerFacetTemplate {
            base_type: "СправочникМенеджер",
            methods: [
                "НайтиПоКоду",
                "НайтиПоНаименованию", 
                "СоздатьЭлемент",
                "Выбрать",
                "ВыбратьИерархически",
            ],
            properties: [],
        },
        
        object: ObjectFacetTemplate {
            base_type: "СправочникОбъект",
            methods: [
                "Записать",
                "Прочитать",
                "ПроверитьЗаполнение",
                "Удалить",
                "ОбменДанными.Загрузка",
            ],
            properties: [
                "Код",
                "Наименование",
                "ПометкаУдаления",
                "Ссылка",
                "ЭтоНовый",
            ],
        },
        
        reference: ReferenceFacetTemplate {
            base_type: "СправочникСсылка",
            methods: [
                "ПолучитьОбъект",
                "Пустая",
                "Метаданные",
            ],
            properties: [
                "Код",           // readonly
                "Наименование",  // readonly
            ],
        },
        
        metadata: MetadataFacetTemplate {
            base_type: "ОбъектМетаданныхСправочник",
            properties: [
                "Имя",
                "Синоним",
                "ДлинаКода",
                "ДлинаНаименования",
                "Иерархический",
                "ВидИерархии",
            ],
            collections: [
                "Реквизиты",
                "ТабличныеЧасти",
                "Формы",
                "Команды",
            ],
        },
    },
    
    document_facets: DocumentFacets { ... },
    register_facets: RegisterFacets { ... },
    // ... другие категории
}
```

### Уровень 2: Конфигурационные типы

Каждый тип из конфигурации хранит ссылки на шаблоны и свои расширения.

```rust
pub struct UnifiedBslType {
    // Идентификация
    core_name: String,              // "Контрагенты"
    metadata_kind: MetadataKind,    // Catalog
    
    // Фасеты - активные представления типа
    facets: TypeFacets {
        manager: Some(ActiveFacet {
            template_ref: FacetRef::CatalogManager,
            type_name: "СправочникМенеджер.Контрагенты",
            access_path: AccessPath::GlobalProperty("Справочники.Контрагенты"),
            extensions: FacetExtensions::None,
        }),
        
        object: Some(ActiveFacet {
            template_ref: FacetRef::CatalogObject,
            type_name: "СправочникОбъект.Контрагенты",
            access_path: AccessPath::MethodResult("СоздатьЭлемент"),
            extensions: FacetExtensions::Properties {
                // Дополнительные свойства из конфигурации
                properties: ["ИНН", "КПП", "ОГРН"],
                tabular_sections: ["КонтактныеЛица", "БанковскиеСчета"],
                // Методы из модуля объекта
                custom_methods: ["ПроверитьИНН", "ЗаполнитьПоИНН"],
            },
        }),
        
        reference: Some(ActiveFacet {
            template_ref: FacetRef::CatalogReference,
            type_name: "СправочникСсылка.Контрагенты",
            access_path: AccessPath::MethodResult("НайтиПоКоду"),
            extensions: FacetExtensions::ReadOnlyProperties {
                properties: ["ИНН", "КПП", "ОГРН"],
            },
        }),
        
        metadata: Some(ActiveFacet {
            template_ref: FacetRef::CatalogMetadata,
            type_name: "ОбъектМетаданныхСправочник",
            access_path: AccessPath::PropertyChain(["Метаданные", "Справочники", "Контрагенты"]),
            extensions: FacetExtensions::None,
        }),
    },
    
    // Правила доступа
    access_rules: AccessRules {
        can_construct: false,        // Нельзя: Новый СправочникМенеджер.Контрагенты
        is_global_property: false,   // Доступ через Справочники
        is_singleton: false,
        available_contexts: vec![Server, ThickClient],
    },
}
```

### Уровень 3: Глобальный контекст

Глобальные функции, свойства и константы - это НЕ типы, а элементы глобального контекста.

```rust
pub struct GlobalContext {
    // Глобальные свойства - точки доступа к типам
    properties: HashMap<String, GlobalProperty>,
    
    // Глобальные функции - прямой вызов
    functions: HashMap<String, GlobalFunction>,
    
    // Глобальные константы
    constants: HashMap<String, GlobalConstant>,
}

pub struct GlobalProperty {
    name: "Справочники",
    returns_type: "СправочникиМенеджер",
    provides_access_to: vec!["СправочникМенеджер.*"],
}

pub struct GlobalFunction {
    name: "СтрНайти",
    english_name: Some("StrFind"),
    parameters: vec![...],
    return_type: Some("Число"),
    // НЕ имеет типа, вызывается напрямую
}

pub struct GlobalConstant {
    name: "Неопределено",
    type_name: "Неопределено",
    value: Undefined,
}
```

## Контекстное разрешение типов

### Определение активного фасета

```rust
pub struct ContextResolver {
    pub fn resolve_facet(&self, type_name: &str, context: &Context) -> Option<ActiveFacet> {
        let unified_type = self.index.find_type(type_name)?;
        
        match context {
            // После точки от глобального свойства
            Context::AfterDot("Справочники") => {
                unified_type.facets.manager
            },
            
            // После точки от метаданных
            Context::AfterDot("Метаданные.Справочники") => {
                unified_type.facets.metadata
            },
            
            // Результат метода создания
            Context::MethodResult("СоздатьЭлемент") => {
                unified_type.facets.object
            },
            
            // Результат поиска
            Context::MethodResult("НайтиПоКоду") => {
                unified_type.facets.reference
            },
            
            // По умолчанию - ссылка (наиболее общий случай)
            _ => unified_type.facets.reference
        }
    }
}
```

### Получение методов с учётом фасета

```rust
impl UnifiedBslIndex {
    pub fn get_methods_for_context(
        &self,
        type_name: &str,
        context: &Context
    ) -> Vec<Method> {
        // 1. Определяем активный фасет
        let facet = self.resolve_facet(type_name, context)?;
        
        // 2. Получаем шаблон фасета
        let template = self.facet_templates.get_template(facet.template_ref)?;
        
        // 3. Собираем методы
        let mut methods = vec![];
        
        // Базовые методы из шаблона
        methods.extend(template.methods.clone());
        
        // Расширения из конфигурации
        if let FacetExtensions::Properties { custom_methods, .. } = &facet.extensions {
            methods.extend(custom_methods.clone());
        }
        
        methods
    }
}
```

## Автодополнение с учётом контекста

### Правила показа элементов

```rust
pub fn get_completions(context: &Context) -> Vec<CompletionItem> {
    match context {
        // Пустой контекст - глобальные элементы
        Context::EmptyLine => {
            vec![]
                .extend(global_context.functions)  // СтрНайти, Сообщить
                .extend(global_context.properties) // Справочники, Метаданные
                .extend(global_context.constants)  // Неопределено, Истина
        },
        
        // После "Новый" - только конструируемые типы
        Context::AfterNew => {
            configuration_types
                .filter(|t| t.access_rules.can_construct)
                .filter(|t| !t.is_manager())  // НЕ показываем менеджеры
                .collect()
        },
        
        // После точки - члены активного фасета
        Context::AfterDot(parent) => {
            let parent_type = resolve_type(parent)?;
            let facet = resolve_facet_for_context(parent_type, context)?;
            get_members_for_facet(facet)
        },
        
        // После "Метаданные.Справочники." - имена справочников
        Context::PropertyChain(["Метаданные", "Справочники"]) => {
            configuration_types
                .filter(|t| t.metadata_kind == MetadataKind::Catalog)
                .map(|t| t.core_name)
                .collect()
        }
    }
}
```

## Преимущества подхода

### 1. Единая точка правды
- Один тип "Контрагенты" вместо 5-6 отдельных
- Изменения в одном месте при рефакторинге
- Понятная связь между представлениями

### 2. Эффективность хранения
- Платформенные методы хранятся один раз в шаблонах
- Нет дублирования данных между типами
- Компактное представление в памяти

### 3. Контекстная корректность
- Показываем только доступные в контексте элементы
- Правильная фильтрация для автодополнения
- Учёт особенностей 1С (менеджеры не конструируются)

### 4. Расширяемость
- Легко добавить новые фасеты
- Простое добавление методов из модулей
- Версионирование платформы через шаблоны

### 5. Производительность
- O(1) поиск типа по имени
- Быстрое определение активного фасета
- Эффективная фильтрация при автодополнении

## Примеры использования

### Пример 1: Автодополнение после точки

```bsl
// Код: Справочники.Контр|
```

1. Определяем контекст: `AfterDot("Справочники")`
2. Справочники - это GlobalProperty, возвращает менеджеры
3. Фильтруем типы с фасетом manager
4. Показываем: "Контрагенты", "Номенклатура", ...

### Пример 2: Определение типа переменной

```bsl
// Код: Контрагент = Справочники.Контрагенты.НайтиПоКоду("123");
```

1. Находим тип "Контрагенты"
2. Активен фасет manager (после Справочники.)
3. Метод НайтиПоКоду возвращает reference фасет
4. Переменная Контрагент получает тип "СправочникСсылка.Контрагенты"

### Пример 3: Метаданные

```bsl
// Код: Для Каждого Реквизит Из Метаданные.Справочники.Контрагенты.Реквизиты Цикл
```

1. Метаданные.Справочники - коллекция метаданных
2. Контрагенты - активируем metadata фасет
3. Реквизиты - коллекция в metadata фасете
4. Тип элемента: "ОбъектМетаданныхРеквизит"

## Специальные случаи

### Singleton типы

```rust
// Метаданные - единственный экземпляр
UnifiedBslType {
    core_name: "Метаданные",
    metadata_kind: MetadataKind::GlobalSingleton,
    facets: {
        singleton: Some(SingletonFacet { ... }),
        // Остальные фасеты не применимы
    }
}
```

### Конструируемые типы с множественными конструкторами

В 1С многие типы имеют несколько вариантов конструкторов:

```bsl
// Примеры множественных конструкторов
ТЗ1 = Новый ТаблицаЗначений;
ТЗ2 = Новый ТаблицаЗначений(Шаблон);

Массив1 = Новый Массив;
Массив2 = Новый Массив(10);
Массив3 = Новый Массив(10, 5);

Структура1 = Новый Структура;
Структура2 = Новый Структура("Имя, Фамилия", "Иван", "Иванов");
```

#### Расширенный ConstructorFacet

```rust
pub struct ConstructorFacet {
    type_name: String,
    
    // Множественные сигнатуры конструктора
    constructors: Vec<ConstructorSignature>,
    
    methods: Vec<Method>,
    properties: Vec<Property>,
}

pub struct ConstructorSignature {
    description: Option<String>,
    parameters: Vec<ConstructorParameter>,
    examples: Vec<String>,
    availability: Vec<BslContext>,
}

pub struct ConstructorParameter {
    name: String,
    type_name: String,
    is_optional: bool,
    default_value: Option<String>,
    description: Option<String>,
}

// Пример: ТаблицаЗначений с двумя конструкторами
UnifiedBslType {
    core_name: "ТаблицаЗначений",
    metadata_kind: MetadataKind::PlatformCollection,
    facets: {
        constructor: Some(ConstructorFacet {
            type_name: "ТаблицаЗначений",
            constructors: vec![
                ConstructorSignature {
                    description: Some("Создает пустую таблицу значений"),
                    parameters: vec![],
                    examples: vec!["Новый ТаблицаЗначений"],
                },
                ConstructorSignature {
                    description: Some("Создает таблицу на основе шаблона"),
                    parameters: vec![
                        ConstructorParameter {
                            name: "Шаблон",
                            type_name: "ТаблицаЗначений",
                            is_optional: false,
                            description: Some("Таблица-шаблон"),
                        },
                    ],
                    examples: vec!["Новый ТаблицаЗначений(ШаблонТаблицы)"],
                },
            ],
            methods: [...],
            properties: [...],
        }),
    }
}
```

#### Автодополнение для множественных конструкторов

```rust
impl CompletionProvider {
    pub fn get_constructor_completions(
        &self,
        type_name: &str,
        partial_params: &[String]
    ) -> Vec<CompletionItem> {
        let unified_type = self.index.find_type(type_name)?;
        
        if let Some(constructor_facet) = &unified_type.facets.constructor {
            // Показываем все варианты конструкторов
            constructor_facet.constructors
                .iter()
                .map(|sig| CompletionItem {
                    label: format_constructor(type_name, sig),
                    detail: sig.description.clone(),
                    insert_text: generate_snippet(type_name, sig),
                    kind: CompletionItemKind::Constructor,
                })
                .collect()
        } else {
            vec![]
        }
    }
}
```

#### Проверка типов для конструкторов

```rust
impl TypeChecker {
    pub fn check_constructor_call(
        &self,
        type_name: &str,
        arguments: &[Expression]
    ) -> Result<TypeInfo> {
        let unified_type = self.index.find_type(type_name)?;
        
        if let Some(constructor_facet) = &unified_type.facets.constructor {
            // Ищем подходящую сигнатуру по количеству и типам аргументов
            for signature in &constructor_facet.constructors {
                if self.arguments_match_signature(arguments, signature) {
                    return Ok(TypeInfo {
                        type_name: type_name.to_string(),
                        facet: FacetKind::Constructor,
                    });
                }
            }
            
            Err(anyhow::anyhow!(
                "Нет подходящего конструктора для {} с аргументами: {:?}",
                type_name, arguments
            ))
        } else {
            Err(anyhow::anyhow!("Тип {} не может быть создан через Новый", type_name))
        }
    }
}
```

### Глобальные функции

```rust
// СтрНайти - НЕ тип, а функция в глобальном контексте
GlobalFunction {
    name: "СтрНайти",
    parameters: [...],
    // Не имеет фасетов, вызывается напрямую
}
```

## Коллекции с типизированными элементами

В 1С многие коллекции имеют строго типизированные элементы, которые часто доступны только для чтения. Это универсальный паттерн, применяемый как в runtime, так и в metadata.

### Примеры коллекций и их элементов

```bsl
// Runtime: Соответствие → КлючИЗначение
Для Каждого Элемент Из Соответствие Цикл
    Сообщить(Элемент.Ключ);      // readonly
    Сообщить(Элемент.Значение);  // readonly
КонецЦикла;

// Metadata: КоллекцияОбъектовМетаданных → ОбъектМетаданныхСправочник
Для Каждого Справочник Из Метаданные.Справочники Цикл
    Сообщить(Справочник.Имя);    // readonly
    Сообщить(Справочник.Синоним); // readonly
КонецЦикла;
```

### Расширение архитектуры для типизированных элементов

#### Информация об элементах коллекции

```rust
pub struct CollectionFacet {
    type_name: String,
    
    // Информация о типе элементов
    element_access: ElementAccessRules,
    
    constructors: Vec<ConstructorSignature>,
    methods: Vec<Method>,
    properties: Vec<Property>,
}

pub struct ElementAccessRules {
    // Тип элемента при итерации
    iteration_type: Option<String>,
    
    // Тип при индексном доступе (может отличаться!)
    index_access_type: Option<String>,
    
    // Типы возврата методов
    method_return_types: HashMap<String, String>,
    
    // Мутабельность элементов
    element_mutability: ElementMutability,
}

pub enum ElementMutability {
    FullyReadOnly,      // КлючИЗначение - все свойства readonly
    ContextDependent,   // СтрокаТаблицыЗначений - зависит от контекста
    FullyMutable,       // Элементы массива - можно изменять
}
```

#### Специальные типы элементов

```rust
// КлючИЗначение - элемент Соответствия
UnifiedBslType {
    core_name: "КлючИЗначение",
    metadata_kind: MetadataKind::CollectionElement,
    
    facets: TypeFacets {
        // Нельзя создать через Новый
        constructor: None,
        
        // Только readonly представление
        readonly_element: Some(ReadOnlyElementFacet {
            type_name: "КлючИЗначение",
            properties: vec![
                Property {
                    name: "Ключ",
                    type_name: "Произвольный",
                    is_readonly: true,
                },
                Property {
                    name: "Значение",
                    type_name: "Произвольный",
                    is_readonly: true,
                },
            ],
            obtainable_from: vec!["Соответствие", "ФиксированноеСоответствие"],
        }),
    },
    
    access_rules: AccessRules {
        can_construct: false,
        is_iteration_element: true,
    },
}
```

### Важное различие: контекстно-зависимые типы возврата

```bsl
// РАЗНЫЕ типы для разных способов доступа к Соответствию

// 1. Итерация возвращает КлючИЗначение
Для Каждого Элемент Из Соответствие Цикл
    // Элемент: КлючИЗначение
    Ключ = Элемент.Ключ;
КонецЦикла;

// 2. Индексный доступ возвращает значение напрямую
Значение = Соответствие["ключ"];  // Произвольный, НЕ КлючИЗначение

// 3. Метод тоже возвращает значение
Значение = Соответствие.Получить("ключ");  // Произвольный
```

#### Конфигурация для разных коллекций

```rust
// Соответствие - разные типы для разных контекстов
CollectionFacet {
    type_name: "Соответствие",
    element_access: ElementAccessRules {
        iteration_type: Some("КлючИЗначение"),
        index_access_type: Some("Произвольный"),  // НЕ КлючИЗначение!
        method_return_types: {
            "Получить": "Произвольный",
            "Удалить": "Неопределено",
        },
        element_mutability: ElementMutability::FullyReadOnly,
    },
}

// Метаданные.Справочники - единообразный тип элементов
CollectionFacet {
    type_name: "КоллекцияОбъектовМетаданных",
    element_access: ElementAccessRules {
        iteration_type: Some("ОбъектМетаданныхСправочник"),
        index_access_type: Some("ОбъектМетаданныхСправочник"),
        method_return_types: {
            "Найти": "ОбъектМетаданныхСправочник",
            "Количество": "Число",
            "Содержит": "Булево",
        },
        element_mutability: ElementMutability::FullyReadOnly,
    },
}

// ТаблицаЗначений - контекстно-зависимая мутабельность
CollectionFacet {
    type_name: "ТаблицаЗначений",
    element_access: ElementAccessRules {
        iteration_type: Some("СтрокаТаблицыЗначений"),
        index_access_type: Some("СтрокаТаблицыЗначений"),
        method_return_types: {
            "Добавить": "СтрокаТаблицыЗначений",
            "Найти": "СтрокаТаблицыЗначений",
            "НайтиСтроки": "Массив",  // Массив из СтрокаТаблицыЗначений
        },
        element_mutability: ElementMutability::ContextDependent,
    },
}
```

### Определение типа в контексте итерации

```rust
impl TypeResolver {
    pub fn resolve_iteration_variable(
        &self,
        collection_type: &str,
        context: &IterationContext
    ) -> Option<TypeInfo> {
        let unified_type = self.index.find_type(collection_type)?;
        
        // Получаем правила доступа к элементам
        if let Some(collection_facet) = self.get_collection_facet(unified_type) {
            if let Some(iteration_type) = &collection_facet.element_access.iteration_type {
                // Определяем мутабельность
                let is_readonly = match collection_facet.element_access.element_mutability {
                    ElementMutability::FullyReadOnly => true,
                    ElementMutability::ContextDependent => {
                        // В цикле Для Каждого обычно readonly
                        context.is_for_each_loop
                    },
                    ElementMutability::FullyMutable => false,
                };
                
                return Some(TypeInfo {
                    type_name: iteration_type.clone(),
                    is_readonly,
                    context: TypeContext::IterationElement,
                });
            }
        }
        
        // Fallback для динамических коллекций
        Some(TypeInfo::arbitrary())
    }
    
    pub fn resolve_index_access(
        &self,
        collection_type: &str,
        index: &Expression
    ) -> Option<TypeInfo> {
        let unified_type = self.index.find_type(collection_type)?;
        
        if let Some(collection_facet) = self.get_collection_facet(unified_type) {
            // Для индексного доступа может быть другой тип!
            if let Some(index_type) = &collection_facet.element_access.index_access_type {
                return Some(TypeInfo {
                    type_name: index_type.clone(),
                    is_readonly: false,  // При индексном доступе часто можно изменять
                    context: TypeContext::IndexAccess,
                });
            }
        }
        
        None
    }
}
```

### Унифицированный подход к коллекциям

Паттерн "коллекция с типизированными элементами" в 1С:

1. **Универсален** - применяется в runtime (Соответствие) и metadata (КоллекцияОбъектовМетаданных)
2. **Контекстно-зависим** - тип возврата зависит от способа доступа (итерация vs индекс vs метод)
3. **Часто readonly** - особенно для metadata и специальных типов (КлючИЗначение)
4. **Не конструируем** - типы элементов часто нельзя создать через Новый

Это позволяет:
- Правильно определять тип переменной в циклах
- Показывать корректные свойства для элементов коллекций
- Учитывать readonly ограничения
- Не показывать типы элементов после "Новый"

## Переходы между фасетами и динамический доступ

### Связи между runtime и metadata мирами

В 1С существует возможность переключения между разными представлениями одного типа:

```bsl
// Получение метаданных из runtime объекта
Контрагент = Справочники.Контрагенты.НайтиПоКоду("123");
МетаданныеКонтрагента = Контрагент.Метаданные();  // ОбъектМетаданныхСправочник

// Создание runtime объекта по метаданным
МетаСправочник = Метаданные.Справочники.Контрагенты;
Менеджер = Справочники[МетаСправочник.Имя];       // СправочникМенеджер.Контрагенты
```

### Расширение архитектуры для поддержки переходов

#### Cross-facet методы

```rust
pub struct FacetTemplate {
    // Существующие поля
    base_type: String,
    methods: Vec<Method>,
    properties: Vec<Property>,
    
    // НОВОЕ: Методы, возвращающие другие фасеты
    cross_facet_methods: Vec<CrossFacetMethod>,
}

pub struct CrossFacetMethod {
    method_name: String,           // "Метаданные"
    returns: FacetTransition {
        target_facet: FacetKind,   // Metadata
        of_same_type: bool,        // true - тот же core_type
    },
}

// Пример для reference facet
ReferenceFacetTemplate {
    base_type: "СправочникСсылка",
    methods: [...],
    cross_facet_methods: vec![
        CrossFacetMethod {
            method_name: "Метаданные",
            returns: FacetTransition {
                target_facet: FacetKind::Metadata,
                of_same_type: true,
            },
        },
    ],
}
```

#### Динамический доступ по индексу

```rust
pub struct GlobalProperty {
    name: String,
    returns_type: String,
    provides_access_to: Vec<String>,
    
    // НОВОЕ: Поддержка индексного доступа
    indexing: Option<IndexingSupport>,
}

pub struct IndexingSupport {
    index_type: IndexType,          // ByName, ByIndex
    resolution: IndexResolution,
}

pub enum IndexResolution {
    // Статический поиск по имени
    StaticLookup {
        collection: HashMap<String, String>,
    },
    
    // Динамический поиск в индексе типов
    DynamicLookup {
        returns_facet: FacetKind,   // Manager для Справочники[имя]
    },
}

// Пример для глобального свойства Справочники
GlobalProperty {
    name: "Справочники",
    returns_type: "СправочникиМенеджер",
    indexing: Some(IndexingSupport {
        index_type: IndexType::ByName,
        resolution: IndexResolution::DynamicLookup {
            returns_facet: FacetKind::Manager,
        },
    }),
}
```

#### Свойства с динамическими значениями

```rust
pub struct PropertyTemplate {
    name: String,
    type_name: String,
    
    // НОВОЕ: Динамическое значение
    value_source: Option<ValueSource>,
}

pub enum ValueSource {
    Static(String),              // Константное значение
    Dynamic(DynamicValue),       // Вычисляемое значение
}

pub enum DynamicValue {
    CoreName,                    // Возвращает core_name типа
    TypeName,                    // Возвращает полное имя типа
    Custom(String),              // Кастомная логика
}

// Пример для metadata facet
MetadataFacetTemplate {
    properties: vec![
        PropertyTemplate {
            name: "Имя",
            type_name: "Строка",
            value_source: Some(ValueSource::Dynamic(DynamicValue::CoreName)),
        },
    ],
}
```

### Реализация переходов между фасетами

```rust
impl ContextResolver {
    // Разрешение метода, возвращающего другой фасет
    pub fn resolve_cross_facet_method(
        &self,
        source_type: &TypeInfo,
        method_name: &str
    ) -> Option<TypeInfo> {
        let unified_type = self.index.find_type(&source_type.core_name)?;
        let source_facet = self.get_active_facet(source_type)?;
        
        // Ищем cross-facet метод
        if let Some(cross_method) = source_facet.find_cross_facet_method(method_name) {
            if cross_method.returns.of_same_type {
                // Возвращаем другой фасет того же типа
                match cross_method.returns.target_facet {
                    FacetKind::Metadata => {
                        return Some(TypeInfo::from_facet(unified_type.facets.metadata));
                    },
                    // ... другие фасеты
                }
            }
        }
        None
    }
    
    // Разрешение динамического доступа по индексу
    pub fn resolve_indexed_access(
        &self,
        collection: &str,
        index_value: &Value
    ) -> Option<TypeInfo> {
        let global_prop = self.global_context.properties.get(collection)?;
        
        if let Some(indexing) = &global_prop.indexing {
            match &indexing.resolution {
                IndexResolution::DynamicLookup { returns_facet } => {
                    if let Value::String(name) = index_value {
                        // Находим тип по имени
                        let unified_type = self.index.find_type(name)?;
                        // Возвращаем указанный фасет
                        return Some(self.get_facet_by_kind(unified_type, *returns_facet));
                    }
                },
                IndexResolution::StaticLookup { collection } => {
                    // Статический поиск в коллекции
                    if let Value::String(key) = index_value {
                        let type_name = collection.get(key)?;
                        return Some(self.resolve_type(type_name));
                    }
                }
            }
        }
        None
    }
}
```

### Примеры использования переходов

#### Пример 1: Метаданные() из runtime объекта

```bsl
НовыйКонтрагент = Справочники.Контрагенты.СоздатьЭлемент();
Мета = НовыйКонтрагент.Метаданные();
```

1. `НовыйКонтрагент` имеет тип с object facet
2. Метод `Метаданные()` - это cross-facet метод
3. Возвращается metadata facet того же типа "Контрагенты"
4. `Мета` получает тип "ОбъектМетаданныхСправочник"

#### Пример 2: Динамический доступ к менеджеру

```bsl
ИмяСправочника = "Контрагенты";
Менеджер = Справочники[ИмяСправочника];
```

1. `Справочники` поддерживает индексный доступ
2. При индексе строкой ищется тип с таким core_name
3. Возвращается manager facet найденного типа
4. `Менеджер` получает тип "СправочникМенеджер.Контрагенты"

## Заключение

Архитектура "Unified Type System with Facets" с расширениями для переходов между фасетами позволяет:

- Корректно представить уникальную систему типов 1С:Предприятие
- Поддерживать переходы между runtime и metadata представлениями
- Обрабатывать динамический доступ по индексу
- Сохранять связность между разными представлениями одного типа
- Обеспечивать правильное автодополнение в любом контексте

Данный подход решает проблему множественных представлений одной сущности и поддерживает все особенности платформы 1С, сохраняя при этом производительность и понятность кода.