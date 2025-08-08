//! Иерархическая организация типов для UI
//! 
//! Модуль предоставляет структуры для организации BSL типов в древовидную иерархию,
//! оптимизированную для отображения в пользовательском интерфейсе (VSCode TreeDataProvider и др.)

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use super::{BslEntity, BslEntityKind};

/// Категория типов в иерархии
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TypeCategory {
    /// Примитивные типы (Строка, Число, Булево, Дата)
    PrimitiveTypes,
    /// Коллекции (Массив, Структура, Соответствие)
    Collections,
    /// Табличные данные (ТаблицаЗначений, ДеревоЗначений)
    TabularData,
    /// Справочники конфигурации
    Catalogs,
    /// Документы конфигурации
    Documents,
    /// Регистры сведений
    InformationRegisters,
    /// Регистры накопления
    AccumulationRegisters,
    /// Планы видов характеристик
    ChartsOfCharacteristicTypes,
    /// Планы счетов
    ChartsOfAccounts,
    /// Планы видов расчета
    ChartsOfCalculationTypes,
    /// Перечисления
    Enums,
    /// Отчеты
    Reports,
    /// Обработки
    DataProcessors,
    /// Бизнес-процессы
    BusinessProcesses,
    /// Задачи
    Tasks,
    /// Общие модули
    CommonModules,
    /// Роли и права
    RolesAndRights,
    /// Системные типы
    SystemTypes,
    /// Глобальный контекст
    GlobalContext,
    /// Формы
    Forms,
    /// Макеты
    Templates,
    /// Прочие типы
    Other,
}

impl TypeCategory {
    /// Возвращает человекочитаемое название категории
    pub fn display_name(&self) -> &str {
        match self {
            TypeCategory::PrimitiveTypes => "Примитивные типы",
            TypeCategory::Collections => "Коллекции",
            TypeCategory::TabularData => "Табличные данные",
            TypeCategory::Catalogs => "Справочники",
            TypeCategory::Documents => "Документы",
            TypeCategory::InformationRegisters => "Регистры сведений",
            TypeCategory::AccumulationRegisters => "Регистры накопления",
            TypeCategory::ChartsOfCharacteristicTypes => "Планы видов характеристик",
            TypeCategory::ChartsOfAccounts => "Планы счетов",
            TypeCategory::ChartsOfCalculationTypes => "Планы видов расчета",
            TypeCategory::Enums => "Перечисления",
            TypeCategory::Reports => "Отчеты",
            TypeCategory::DataProcessors => "Обработки",
            TypeCategory::BusinessProcesses => "Бизнес-процессы",
            TypeCategory::Tasks => "Задачи",
            TypeCategory::CommonModules => "Общие модули",
            TypeCategory::RolesAndRights => "Роли и права",
            TypeCategory::SystemTypes => "Системные типы",
            TypeCategory::GlobalContext => "Глобальный контекст",
            TypeCategory::Forms => "Формы",
            TypeCategory::Templates => "Макеты",
            TypeCategory::Other => "Прочие",
        }
    }
    
    /// Возвращает иконку для категории (для VSCode)
    pub fn icon(&self) -> &str {
        match self {
            TypeCategory::PrimitiveTypes => "symbol-number",
            TypeCategory::Collections => "symbol-array",
            TypeCategory::TabularData => "table",
            TypeCategory::Catalogs => "book",
            TypeCategory::Documents => "file-text",
            TypeCategory::InformationRegisters => "database",
            TypeCategory::AccumulationRegisters => "graph",
            TypeCategory::ChartsOfCharacteristicTypes => "list-tree",
            TypeCategory::ChartsOfAccounts => "credit-card",
            TypeCategory::ChartsOfCalculationTypes => "calculator",
            TypeCategory::Enums => "symbol-enum",
            TypeCategory::Reports => "graph-line",
            TypeCategory::DataProcessors => "gear",
            TypeCategory::BusinessProcesses => "workflow",
            TypeCategory::Tasks => "checklist",
            TypeCategory::CommonModules => "symbol-module",
            TypeCategory::RolesAndRights => "shield",
            TypeCategory::SystemTypes => "settings",
            TypeCategory::GlobalContext => "globe",
            TypeCategory::Forms => "window",
            TypeCategory::Templates => "file-code",
            TypeCategory::Other => "symbol-misc",
        }
    }
    
    /// Определяет категорию на основе BslEntityKind
    pub fn from_entity_kind(kind: &BslEntityKind) -> Self {
        match kind {
            BslEntityKind::Primitive => TypeCategory::PrimitiveTypes,
            BslEntityKind::Array | BslEntityKind::Structure | BslEntityKind::Map => TypeCategory::Collections,
            BslEntityKind::ValueList => TypeCategory::Collections,
            BslEntityKind::ValueTable | BslEntityKind::ValueTree => TypeCategory::TabularData,
            BslEntityKind::Catalog => TypeCategory::Catalogs,
            BslEntityKind::Document => TypeCategory::Documents,
            BslEntityKind::InformationRegister => TypeCategory::InformationRegisters,
            BslEntityKind::AccumulationRegister => TypeCategory::AccumulationRegisters,
            BslEntityKind::ChartOfCharacteristicTypes => TypeCategory::ChartsOfCharacteristicTypes,
            BslEntityKind::ChartOfAccounts => TypeCategory::ChartsOfAccounts,
            BslEntityKind::ChartOfCalculationTypes => TypeCategory::ChartsOfCalculationTypes,
            BslEntityKind::Enum => TypeCategory::Enums,
            BslEntityKind::Report => TypeCategory::Reports,
            BslEntityKind::DataProcessor => TypeCategory::DataProcessors,
            BslEntityKind::BusinessProcess => TypeCategory::BusinessProcesses,
            BslEntityKind::Task => TypeCategory::Tasks,
            BslEntityKind::CommonModule => TypeCategory::CommonModules,
            BslEntityKind::Global => TypeCategory::GlobalContext,
            BslEntityKind::System => TypeCategory::SystemTypes,
            BslEntityKind::Form => TypeCategory::Forms,
            _ => TypeCategory::Other,
        }
    }
}

/// Узел в иерархии типов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeNode {
    /// Уникальный идентификатор узла
    pub id: String,
    /// Отображаемое имя узла
    pub label: String,
    /// Тип узла (категория или конкретный тип)
    pub node_type: TypeNodeType,
    /// Дочерние узлы
    pub children: Vec<TypeNode>,
    /// Количество элементов в узле (для категорий)
    pub item_count: usize,
    /// Иконка для отображения
    pub icon: String,
    /// Развернут ли узел по умолчанию
    pub expanded: bool,
    /// Дополнительные метаданные
    pub metadata: HashMap<String, String>,
}

/// Тип узла в иерархии
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeNodeType {
    /// Корневой узел
    Root,
    /// Категория типов
    Category(TypeCategory),
    /// Конкретный тип
    Type(String), // qualified_name типа
    /// Группа (для дополнительной группировки)
    Group(String),
}

/// Иерархическая структура типов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeHierarchy {
    /// Корневой узел дерева
    pub root: TypeNode,
    /// Индекс узлов по ID для быстрого доступа
    #[serde(skip)]
    node_index: HashMap<String, TypeNode>,
    /// Статистика по категориям
    pub statistics: HashMap<TypeCategory, CategoryStatistics>,
}

/// Статистика по категории
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStatistics {
    /// Общее количество типов
    pub total_types: usize,
    /// Количество методов во всех типах категории
    pub total_methods: usize,
    /// Количество свойств во всех типах категории
    pub total_properties: usize,
    /// Наиболее часто используемые типы
    pub top_types: Vec<String>,
}

impl TypeHierarchy {
    /// Создает новую иерархию из списка сущностей
    pub fn from_entities(entities: &[BslEntity]) -> Self {
        let mut hierarchy = Self {
            root: TypeNode {
                id: "root".to_string(),
                label: "BSL Type System".to_string(),
                node_type: TypeNodeType::Root,
                children: Vec::new(),
                item_count: entities.len(),
                icon: "symbol-namespace".to_string(),
                expanded: true,
                metadata: HashMap::new(),
            },
            node_index: HashMap::new(),
            statistics: HashMap::new(),
        };
        
        // Группируем сущности по категориям
        let mut categories: HashMap<TypeCategory, Vec<&BslEntity>> = HashMap::new();
        
        for entity in entities {
            let category = TypeCategory::from_entity_kind(&entity.entity_kind);
            categories.entry(category).or_insert_with(Vec::new).push(entity);
        }
        
        // Создаем узлы категорий
        for (category, category_entities) in categories {
            if category_entities.is_empty() {
                continue;
            }
            
            let category_node = hierarchy.create_category_node(&category, &category_entities);
            hierarchy.root.children.push(category_node);
            
            // Собираем статистику
            let stats = CategoryStatistics {
                total_types: category_entities.len(),
                total_methods: category_entities.iter()
                    .map(|e| e.interface.methods.len())
                    .sum(),
                total_properties: category_entities.iter()
                    .map(|e| e.interface.properties.len())
                    .sum(),
                top_types: category_entities.iter()
                    .take(5)
                    .map(|e| e.qualified_name.clone())
                    .collect(),
            };
            hierarchy.statistics.insert(category, stats);
        }
        
        // Сортируем категории по приоритету отображения
        let priority_map = hierarchy.create_priority_map();
        hierarchy.root.children.sort_by_key(|node| {
            if let TypeNodeType::Category(cat) = &node.node_type {
                priority_map.get(cat).copied().unwrap_or(999)
            } else {
                999
            }
        });
        
        hierarchy
    }
    
    /// Создает узел категории
    fn create_category_node(&self, category: &TypeCategory, entities: &[&BslEntity]) -> TypeNode {
        let mut node = TypeNode {
            id: format!("category_{:?}", category),
            label: format!("{} ({})", category.display_name(), entities.len()),
            node_type: TypeNodeType::Category(category.clone()),
            children: Vec::new(),
            item_count: entities.len(),
            icon: category.icon().to_string(),
            expanded: false,
            metadata: HashMap::new(),
        };
        
        // Добавляем узлы типов
        for entity in entities {
            let type_node = self.create_type_node(entity);
            node.children.push(type_node);
        }
        
        // Сортируем типы по имени
        node.children.sort_by(|a, b| a.label.cmp(&b.label));
        
        node
    }
    
    /// Создает узел типа
    fn create_type_node(&self, entity: &BslEntity) -> TypeNode {
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), format!("{:?}", entity.entity_type));
        metadata.insert("kind".to_string(), format!("{:?}", entity.entity_kind));
        
        if let Some(doc) = &entity.documentation {
            metadata.insert("documentation".to_string(), doc.clone());
        }
        
        TypeNode {
            id: entity.id.0.clone(),
            label: entity.display_name.clone(),
            node_type: TypeNodeType::Type(entity.qualified_name.clone()),
            children: Vec::new(), // Можно добавить методы и свойства как дочерние узлы
            item_count: entity.interface.methods.len() + entity.interface.properties.len(),
            icon: self.get_type_icon(entity),
            expanded: false,
            metadata,
        }
    }
    
    /// Определяет иконку для типа
    fn get_type_icon(&self, entity: &BslEntity) -> String {
        match entity.entity_kind {
            BslEntityKind::Primitive => "symbol-number",
            BslEntityKind::Array => "symbol-array",
            BslEntityKind::Structure => "symbol-structure",
            BslEntityKind::Map => "symbol-namespace",
            BslEntityKind::ValueTable => "table",
            BslEntityKind::Catalog => "book",
            BslEntityKind::Document => "file-text",
            BslEntityKind::Report => "graph-line",
            BslEntityKind::CommonModule => "symbol-module",
            _ => "symbol-misc",
        }.to_string()
    }
    
    /// Создает карту приоритетов категорий
    fn create_priority_map(&self) -> HashMap<TypeCategory, usize> {
        let mut map = HashMap::new();
        map.insert(TypeCategory::PrimitiveTypes, 1);
        map.insert(TypeCategory::Collections, 2);
        map.insert(TypeCategory::TabularData, 3);
        map.insert(TypeCategory::GlobalContext, 4);
        map.insert(TypeCategory::Catalogs, 10);
        map.insert(TypeCategory::Documents, 11);
        map.insert(TypeCategory::InformationRegisters, 12);
        map.insert(TypeCategory::AccumulationRegisters, 13);
        map.insert(TypeCategory::Enums, 20);
        map.insert(TypeCategory::CommonModules, 30);
        map.insert(TypeCategory::Reports, 40);
        map.insert(TypeCategory::DataProcessors, 41);
        map.insert(TypeCategory::Forms, 50);
        map.insert(TypeCategory::SystemTypes, 90);
        map.insert(TypeCategory::Other, 99);
        map
    }
    
    /// Приоритет категории для сортировки
    fn category_priority(&self, category: &TypeCategory) -> usize {
        match category {
            TypeCategory::PrimitiveTypes => 1,
            TypeCategory::Collections => 2,
            TypeCategory::TabularData => 3,
            TypeCategory::GlobalContext => 4,
            TypeCategory::Catalogs => 10,
            TypeCategory::Documents => 11,
            TypeCategory::InformationRegisters => 12,
            TypeCategory::AccumulationRegisters => 13,
            TypeCategory::Enums => 20,
            TypeCategory::CommonModules => 30,
            TypeCategory::Reports => 40,
            TypeCategory::DataProcessors => 41,
            TypeCategory::Forms => 50,
            TypeCategory::SystemTypes => 90,
            TypeCategory::Other => 99,
            _ => 60,
        }
    }
    
    /// Поиск узла по ID
    pub fn find_node(&self, id: &str) -> Option<&TypeNode> {
        self.find_node_recursive(&self.root, id)
    }
    
    fn find_node_recursive<'a>(&self, node: &'a TypeNode, id: &str) -> Option<&'a TypeNode> {
        if node.id == id {
            return Some(node);
        }
        
        for child in &node.children {
            if let Some(found) = self.find_node_recursive(child, id) {
                return Some(found);
            }
        }
        
        None
    }
    
    /// Фильтрация иерархии по запросу
    pub fn filter(&self, query: &str) -> TypeHierarchy {
        let mut filtered_root = self.root.clone();
        filtered_root.children.clear();
        
        let query_lower = query.to_lowercase();
        
        for category_node in &self.root.children {
            let mut filtered_category = category_node.clone();
            filtered_category.children.clear();
            
            // Фильтруем типы в категории
            for type_node in &category_node.children {
                if type_node.label.to_lowercase().contains(&query_lower) {
                    filtered_category.children.push(type_node.clone());
                }
            }
            
            // Добавляем категорию только если есть совпадения
            if !filtered_category.children.is_empty() {
                filtered_category.label = format!(
                    "{} ({})", 
                    self.get_category_name(&filtered_category),
                    filtered_category.children.len()
                );
                filtered_category.item_count = filtered_category.children.len();
                filtered_root.children.push(filtered_category);
            }
        }
        
        TypeHierarchy {
            root: filtered_root,
            node_index: HashMap::new(),
            statistics: self.statistics.clone(),
        }
    }
    
    /// Получает имя категории из узла
    fn get_category_name(&self, node: &TypeNode) -> String {
        if let TypeNodeType::Category(cat) = &node.node_type {
            cat.display_name().to_string()
        } else {
            node.label.clone()
        }
    }
    
    /// Экспортирует иерархию в формат для VSCode TreeDataProvider
    pub fn to_vscode_tree_items(&self) -> Vec<serde_json::Value> {
        self.node_to_vscode_items(&self.root)
    }
    
    fn node_to_vscode_items(&self, node: &TypeNode) -> Vec<serde_json::Value> {
        use serde_json::json;
        
        let mut items = Vec::new();
        
        for child in &node.children {
            let collapsible_state = if child.children.is_empty() {
                "none"
            } else if child.expanded {
                "expanded"
            } else {
                "collapsed"
            };
            
            items.push(json!({
                "id": child.id,
                "label": child.label,
                "collapsibleState": collapsible_state,
                "iconPath": child.icon,
                "contextValue": format!("{:?}", child.node_type),
                "tooltip": child.metadata.get("documentation").cloned().unwrap_or_default(),
                "command": if child.children.is_empty() {
                    json!({
                        "command": "bsl.showTypeInfo",
                        "title": "Show Type Info",
                        "arguments": [child.id]
                    })
                } else {
                    json!(null)
                }
            }));
        }
        
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_category_from_kind() {
        assert_eq!(TypeCategory::from_entity_kind(&BslEntityKind::Primitive), TypeCategory::PrimitiveTypes);
        assert_eq!(TypeCategory::from_entity_kind(&BslEntityKind::Array), TypeCategory::Collections);
        assert_eq!(TypeCategory::from_entity_kind(&BslEntityKind::Catalog), TypeCategory::Catalogs);
    }
    
    #[test]
    fn test_category_display_name() {
        assert_eq!(TypeCategory::PrimitiveTypes.display_name(), "Примитивные типы");
        assert_eq!(TypeCategory::Catalogs.display_name(), "Справочники");
    }
}