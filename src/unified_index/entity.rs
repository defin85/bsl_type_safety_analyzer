use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// <entity-type>
///   <name>BslEntityId</name>
///   <purpose>Уникальный идентификатор BSL сущности</purpose>
///   <usage>
/// ```text
/// &lt;example>
/// ```rust,ignore
/// let id = BslEntityId("Справочники.Номенклатура".to_string());
/// ```
///
/// ```
/// ```
///   </usage>
/// </entity-type>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BslEntityId(pub String);

/// <entity-type>
///   <name>BslEntityType</name>
///   <purpose>Источник определения BSL типа</purpose>
///   <variants>
///     <variant name="Platform">Встроенные типы платформы 1С</variant>
///     <variant name="Configuration">Объекты конфигурации (справочники, документы)</variant>
///     <variant name="Form">Формы объектов</variant>
///     <variant name="Module">Модули с экспортными методами</variant>
///   </variants>
/// </entity-type>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BslEntityType {
    Platform,
    Configuration,
    Form,
    Module,
}

/// <entity-type>
///   <name>BslEntityKind</name>
///   <purpose>Конкретный вид BSL сущности</purpose>
///   <categories>
///     <category name="Primitives">
///       <description>Примитивные типы: Число, Строка, Дата, Булево</description>
///     </category>
///     <category name="Collections">
///       <description>Коллекции: Массив, Структура, Соответствие, СписокЗначений</description>
///     </category>
///     <category name="ConfigurationObjects">
///       <description>Объекты конфигурации: справочники, документы, регистры</description>
///     </category>
///     <category name="Forms">
///       <description>Формы: управляемые и обычные</description>
///     </category>
///     <category name="Modules">
///       <description>Модули различного назначения</description>
///     </category>
///   </categories>
/// </entity-type>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BslEntityKind {
    // Примитивные типы
    Primitive,

    // Коллекции
    Array,
    Structure,
    Map,
    ValueList,
    ValueTable,
    ValueTree,

    // Объекты конфигурации
    Catalog,
    Document,
    ChartOfCharacteristicTypes,
    ChartOfAccounts,
    ChartOfCalculationTypes,
    InformationRegister,
    AccumulationRegister,
    AccountingRegister,
    CalculationRegister,
    BusinessProcess,
    Task,
    ExchangePlan,
    Constant,
    Enum,
    Report,
    DataProcessor,
    DocumentJournal,

    // Формы
    Form,
    ManagedForm,
    OrdinaryForm,

    // Прочие
    CommonModule,
    SessionModule,
    ApplicationModule,
    ExternalConnectionModule,
    ManagedApplicationModule,
    OrdinaryApplicationModule,
    CommandModule,
    ObjectModule,
    ManagerModule,
    RecordSetModule,
    ValueManagerModule,
    TabularSectionManagerModule,

    // Системные
    System,
    Global,
    GlobalFunction,
    GlobalProperty,

    // Другие
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BslEntitySource {
    HBK { version: String },
    ConfigurationXml { path: String },
    FormXml { path: String },
    Module { path: String },
    TextReport { path: String },
    Synthetic, // Программно созданные типы (примитивные типы BSL)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BslApplicationMode {
    /// Обычное приложение (8.1) - обычные формы, нет директив компиляции
    OrdinaryApplication,
    /// Управляемое приложение (8.2+) - управляемые формы, директивы &НаСервере и т.д.
    ManagedApplication,
    /// Смешанный режим - поддержка обоих типов форм
    MixedMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BslContext {
    Client,
    Server,
    ExternalConnection,
    MobileApp,
    MobileClient,
    MobileServer,
    ThickClient,
    ThinClient,
    WebClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslMethod {
    pub name: String,
    pub english_name: Option<String>,
    pub parameters: Vec<BslParameter>,
    pub return_type: Option<String>,
    pub documentation: Option<String>,
    pub availability: Vec<BslContext>,
    pub is_function: bool,
    pub is_export: bool,
    pub is_deprecated: bool,
    pub deprecation_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslParameter {
    pub name: String,
    pub type_name: Option<String>,
    pub is_optional: bool,
    pub default_value: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslProperty {
    pub name: String,
    pub english_name: Option<String>,
    pub type_name: String,
    pub is_readonly: bool,
    pub is_indexed: bool,
    pub documentation: Option<String>,
    pub availability: Vec<BslContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslEvent {
    pub name: String,
    pub parameters: Vec<BslParameter>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BslInterface {
    pub methods: HashMap<String, BslMethod>,
    pub properties: HashMap<String, BslProperty>,
    pub events: HashMap<String, BslEvent>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BslConstraints {
    pub parent_types: Vec<String>,
    pub implements: Vec<String>,
    pub string_length: Option<u32>,
    pub number_precision: Option<(u8, u8)>,
    pub date_fractions: Option<String>,
    pub allowed_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BslRelationships {
    pub owner: Option<String>,
    pub tabular_sections: Vec<BslTabularSection>,
    pub attributes: Vec<String>,
    pub forms: Vec<String>,
    pub commands: Vec<String>,
    pub referenced_by: Vec<String>,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslTabularSection {
    pub name: String,
    pub display_name: String,
    pub attributes: Vec<BslProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslLifecycle {
    pub introduced_version: Option<String>,
    pub deprecated_version: Option<String>,
    pub removed_version: Option<String>,
    pub replacement: Option<String>,
}

/// <entity-type>
///   <name>BslEntity</name>
///   <purpose>Универсальное представление любого BSL типа в единой системе</purpose>
///   <description>
///     Центральная структура данных, представляющая любой тип в BSL:
///     платформенные типы, объекты конфигурации, формы и модули.
///   </description>
///   <usage>
/// &lt;example>
/// ```rust,ignore
/// // Поиск типа в индексе
/// let entity = index.find_entity("Справочники.Номенклатура")?;
///
/// // Проверка наличия метода
/// if entity.has_method("Записать") {
///     let method = &entity.interface.methods["Записать"];
/// }
///
/// // Получение всех свойств
/// let properties = entity.get_all_property_names();
/// ```
/// ```
///   </usage>
///   <fields>
///     <field name="id">Уникальный идентификатор</field>
///     <field name="interface">Методы, свойства и события</field>
///     <field name="constraints">Ограничения типа и наследование</field>
///     <field name="relationships">Связи с другими типами</field>
///   </fields>
/// </entity-type>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslEntity {
    // Идентификация
    pub id: BslEntityId,
    pub qualified_name: String,
    pub display_name: String,
    pub english_name: Option<String>,

    // Классификация
    pub entity_type: BslEntityType,
    pub entity_kind: BslEntityKind,
    pub source: BslEntitySource,

    // Поведение
    pub interface: BslInterface,
    pub constraints: BslConstraints,
    pub relationships: BslRelationships,

    // Метаданные
    pub documentation: Option<String>,
    pub availability: Vec<BslContext>,
    pub lifecycle: BslLifecycle,

    // Флаги доступности (для правильной работы автодополнения)
    pub can_create_with_new: bool,
    pub is_global_property: bool,
    pub is_global_function: bool,
    pub parent_manager: Option<String>,

    // Расширенные данные для специфичной информации
    pub extended_data: serde_json::Map<String, serde_json::Value>,
}

impl BslEntity {
    pub fn new(
        id: String,
        qualified_name: String,
        entity_type: BslEntityType,
        entity_kind: BslEntityKind,
    ) -> Self {
        Self {
            id: BslEntityId(id),
            qualified_name: qualified_name.clone(),
            display_name: qualified_name
                .split('.')
                .next_back()
                .unwrap_or(&qualified_name)
                .to_string(),
            english_name: None,
            entity_type,
            entity_kind,
            source: BslEntitySource::ConfigurationXml {
                path: String::new(),
            },
            interface: BslInterface::default(),
            constraints: BslConstraints::default(),
            relationships: BslRelationships::default(),
            documentation: None,
            availability: vec![],
            lifecycle: BslLifecycle {
                introduced_version: None,
                deprecated_version: None,
                removed_version: None,
                replacement: None,
            },
            can_create_with_new: false,
            is_global_property: false,
            is_global_function: false,
            parent_manager: None,
            extended_data: serde_json::Map::new(),
        }
    }

    pub fn get_all_method_names(&self) -> Vec<&str> {
        self.interface.methods.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_all_property_names(&self) -> Vec<&str> {
        self.interface
            .properties
            .keys()
            .map(|s| s.as_str())
            .collect()
    }

    pub fn has_method(&self, method_name: &str) -> bool {
        self.interface.methods.contains_key(method_name)
    }

    pub fn has_property(&self, property_name: &str) -> bool {
        self.interface.properties.contains_key(property_name)
    }
}
