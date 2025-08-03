use std::collections::HashMap;
use anyhow::Result;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use serde::{Serialize, Deserialize};

use super::entity::{BslEntity, BslEntityId, BslEntityType, BslEntityKind, BslMethod, BslProperty, BslApplicationMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BslLanguagePreference {
    /// Приоритет русским именам (по умолчанию для российских проектов)
    Russian,
    /// Приоритет английским именам (для международных проектов)
    English,
    /// Автоматическое определение по первому найденному типу
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedBslIndex {
    // Режим приложения
    application_mode: BslApplicationMode,
    
    // Основное хранилище
    entities: HashMap<BslEntityId, BslEntity>,
    
    // Индексы поиска O(1)
    by_name: HashMap<String, BslEntityId>,
    by_qualified_name: HashMap<String, BslEntityId>,
    by_type: HashMap<BslEntityType, Vec<BslEntityId>>,
    by_kind: HashMap<BslEntityKind, Vec<BslEntityId>>,
    
    // Специализированные индексы
    methods_by_name: HashMap<String, Vec<BslEntityId>>,
    properties_by_name: HashMap<String, Vec<BslEntityId>>,
    
    // Альтернативные имена для быстрого поиска O(1)
    alternative_names: HashMap<String, BslEntityId>,
    
    // Глобальные алиасы 1С: глобальная переменная -> реальный тип платформы
    global_aliases: HashMap<String, BslEntityId>,
    
    // Языковые индексы для оптимизированного поиска
    russian_names: HashMap<String, BslEntityId>,  // только русские имена
    english_names: HashMap<String, BslEntityId>,  // только английские имена
    
    // Графы отношений
    #[serde(skip)]
    inheritance_graph: DiGraph<BslEntityId, ()>,
    #[serde(skip)]
    inheritance_node_map: HashMap<BslEntityId, NodeIndex>,
    #[serde(skip)]
    reference_graph: DiGraph<BslEntityId, String>,
    #[serde(skip)]
    reference_node_map: HashMap<BslEntityId, NodeIndex>,
}

impl UnifiedBslIndex {
    pub fn new() -> Self {
        Self::with_application_mode(BslApplicationMode::ManagedApplication)
    }
    
    pub fn with_application_mode(mode: BslApplicationMode) -> Self {
        Self {
            application_mode: mode,
            entities: HashMap::new(),
            by_name: HashMap::new(),
            by_qualified_name: HashMap::new(),
            by_type: HashMap::new(),
            by_kind: HashMap::new(),
            methods_by_name: HashMap::new(),
            properties_by_name: HashMap::new(),
            alternative_names: HashMap::new(),
            global_aliases: HashMap::new(),
            russian_names: HashMap::new(),
            english_names: HashMap::new(),
            inheritance_graph: DiGraph::new(),
            inheritance_node_map: HashMap::new(),
            reference_graph: DiGraph::new(),
            reference_node_map: HashMap::new(),
        }
    }
    
    pub fn get_application_mode(&self) -> BslApplicationMode {
        self.application_mode
    }
    
    pub fn add_entity(&mut self, entity: BslEntity) -> Result<()> {
        let id = entity.id.clone();
        let name = entity.display_name.clone();
        let qualified_name = entity.qualified_name.clone();
        let entity_type = entity.entity_type.clone();
        let entity_kind = entity.entity_kind.clone();
        
        // Индексируем методы
        for method_name in entity.interface.methods.keys() {
            self.methods_by_name
                .entry(method_name.clone())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }
        
        // Индексируем свойства
        for property_name in entity.interface.properties.keys() {
            self.properties_by_name
                .entry(property_name.clone())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }
        
        // Добавляем в основные индексы
        self.by_name.insert(name.clone(), id.clone());
        self.by_qualified_name.insert(qualified_name.clone(), id.clone());
        
        // Добавляем альтернативные имена
        self.add_alternative_names(&name, &id);
        
        self.by_type
            .entry(entity_type)
            .or_insert_with(Vec::new)
            .push(id.clone());
            
        self.by_kind
            .entry(entity_kind)
            .or_insert_with(Vec::new)
            .push(id.clone());
        
        // Добавляем узел в графы
        let node_idx = self.inheritance_graph.add_node(id.clone());
        self.inheritance_node_map.insert(id.clone(), node_idx);
        
        let ref_node_idx = self.reference_graph.add_node(id.clone());
        self.reference_node_map.insert(id.clone(), ref_node_idx);
        
        // ООП-подход: Автоматическое наследование методов менеджеров
        let mut enhanced_entity = entity;
        if enhanced_entity.entity_type == BslEntityType::Configuration {
            enhanced_entity = self.inherit_manager_methods(enhanced_entity)?;
        }
        
        // Сохраняем сущность
        self.entities.insert(id, enhanced_entity);
        
        Ok(())
    }
    
    pub fn build_inheritance_relationships(&mut self) -> Result<()> {
        let entities_snapshot: Vec<(BslEntityId, Vec<String>)> = self.entities
            .iter()
            .map(|(id, entity)| (id.clone(), entity.constraints.parent_types.clone()))
            .collect();
            
        for (child_id, parent_names) in entities_snapshot {
            if let Some(&child_node) = self.inheritance_node_map.get(&child_id) {
                for parent_name in parent_names {
                    if let Some(parent_id) = self.by_qualified_name.get(&parent_name).or_else(|| self.by_name.get(&parent_name)) {
                        if let Some(&parent_node) = self.inheritance_node_map.get(parent_id) {
                            self.inheritance_graph.add_edge(parent_node, child_node, ());
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Инициализирует глобальные алиасы 1С для общих объектов платформы
    pub fn initialize_global_aliases(&mut self) -> Result<()> {
        // Маппинг глобальных алиасов 1С к реальным типам платформы
        let aliases = [
            // Пользователи информационной базы
            ("ПользователиИнформационнойБазы", "МенеджерПользователейИнформационнойБазы (InfoBaseUsersManager)"),
            ("InfoBaseUsers", "МенеджерПользователейИнформационнойБазы (InfoBaseUsersManager)"),
            
            // Метаданные (глобальный объект)
            ("Метаданные", "МенеджерМетаданных"),
            ("Metadata", "MetadataManager"),
            
            // Константы
            ("Константы", "МенеджерКонстант"),
            ("Constants", "ConstantsManager"),
            
            // Справочники (общий доступ через СправочникиМенеджер)
            ("Справочники", "СправочникиМенеджер (CatalogsManager)"),
            ("Catalogs", "СправочникиМенеджер (CatalogsManager)"),
            
            // Документы
            ("Документы", "МенеджерДокументов"),
            ("Documents", "DocumentsManager"),
            
            // Регистры сведений
            ("РегистрыСведений", "МенеджерРегистровСведений"),
            ("InformationRegisters", "InformationRegistersManager"),
            
            // Регистры накопления
            ("РегистрыНакопления", "МенеджерРегистровНакопления"),
            ("AccumulationRegisters", "AccumulationRegistersManager"),
            
            // Обработки
            ("Обработки", "МенеджерОбработок"),
            ("DataProcessors", "DataProcessorsManager"),
            
            // Отчеты
            ("Отчеты", "МенеджерОтчетов"),
            ("Reports", "ReportsManager"),
        ];
        
        // Создаем алиасы для найденных типов
        for (alias, target_type) in &aliases {
            if let Some(target_id) = self.find_target_for_alias(target_type) {
                self.global_aliases.insert(alias.to_string(), target_id.clone());
                println!("🔗 Создан глобальный алиас: {} → {}", alias, target_type);
            } else {
                println!("⚠️ Целевой тип '{}' для алиаса '{}' не найден", target_type, alias);
            }
        }
        
        println!("✅ Инициализировано {} глобальных алиасов 1С", self.global_aliases.len());
        Ok(())
    }
    
    /// Ищет целевой тип для алиаса с различными стратегиями поиска
    fn find_target_for_alias(&self, target_type: &str) -> Option<&BslEntityId> {
        println!("🔍 Поиск target для алиаса: '{}'", target_type);
        
        // 1. Точный поиск по qualified_name
        if let Some(id) = self.by_qualified_name.get(target_type) {
            println!("✅ Найден по qualified_name: {}", target_type);
            return Some(id);
        }
        
        // 2. Поиск по display_name
        if let Some(id) = self.by_name.get(target_type) {
            if let Some(entity) = self.entities.get(id) {
                println!("✅ Найден по display_name: {} -> {} ({:?})", 
                    target_type, entity.qualified_name, entity.entity_kind);
                return Some(id);
            }
        }
        
        // 3. Поиск по альтернативным именам
        if let Some(id) = self.alternative_names.get(target_type) {
            println!("✅ Найден по альтернативным именам: {}", target_type);
            return Some(id);
        }
        
        // 4. Гибкий поиск с частичным совпадением (для сложных имен) - только если точно не найдено
        for (qualified_name, id) in &self.by_qualified_name {
            if qualified_name.contains(target_type) {
                println!("✅ Найден по частичному совпадению: {} -> {}", target_type, qualified_name);
                return Some(id);
            }
        }
        
        println!("❌ Тип '{}' не найден", target_type);
        None
    }
    
    /// Парсит display name на составные части
    fn parse_display_name(display_name: &str) -> (Option<String>, Option<String>) {
        if let Some(pos) = display_name.find(" (") {
            if let Some(end_pos) = display_name.rfind(")") {
                let first_name = display_name[..pos].to_string();
                let second_name = display_name[pos + 2..end_pos].to_string();
                
                // Возвращаем оба названия - не определяем язык, просто даем альтернативы
                return (Some(first_name), Some(second_name));
            }
        }
        
        // Если нет скобок, возвращаем только одно название
        (Some(display_name.to_string()), None)
    }
    
    /// Определяет, содержит ли строка кириллические символы
    fn contains_cyrillic(text: &str) -> bool {
        text.chars().any(|c| '\u{0400}' <= c && c <= '\u{04FF}')
    }
    
    /// Добавляет альтернативные имена для быстрого поиска с языковой категоризацией
    fn add_alternative_names(&mut self, display_name: &str, entity_id: &BslEntityId) {
        let (first_name, second_name) = Self::parse_display_name(display_name);
        
        // Добавляем первое название, если оно отличается от полного
        if let Some(name) = first_name {
            if name != *display_name {
                self.alternative_names.insert(name.clone(), entity_id.clone());
                
                // Категоризируем по языку
                if Self::contains_cyrillic(&name) {
                    self.russian_names.insert(name, entity_id.clone());
                } else {
                    self.english_names.insert(name, entity_id.clone());
                }
            }
        }
        
        // Добавляем второе название, если оно есть и отличается от полного
        if let Some(name) = second_name {
            if name != *display_name {
                self.alternative_names.insert(name.clone(), entity_id.clone());
                
                // Категоризируем по языку
                if Self::contains_cyrillic(&name) {
                    self.russian_names.insert(name, entity_id.clone());
                } else {
                    self.english_names.insert(name, entity_id.clone());
                }
            }
        }
    }
    
    /// <api-method>
    ///   <name>find_entity</name>
    ///   <purpose>Поиск сущности по имени с автоопределением языка</purpose>
    ///   <parameters>
    ///     <param name="name" type="&str">Имя сущности для поиска (русское или английское)</param>
    ///   </parameters>
    ///   <returns>Option<&BslEntity></returns>
    ///   <examples>
    ///     <example lang="rust">
    ///       // Поиск платформенного типа
    ///       let entity = index.find_entity("Массив")?;
    ///       let entity = index.find_entity("Array")?; // английский вариант
    ///       
    ///       // Поиск объекта конфигурации
    ///       let entity = index.find_entity("Справочники.Номенклатура")?;
    ///     </example>
    ///   </examples>
    /// </api-method>
    pub fn find_entity(&self, name: &str) -> Option<&BslEntity> {
        self.find_entity_with_preference(name, BslLanguagePreference::Auto)
    }
    
    /// <api-method>
    ///   <name>find_entity_with_preference</name>
    ///   <purpose>Поиск сущности с указанием языковых предпочтений</purpose>
    ///   <parameters>
    ///     <param name="name" type="&str">Имя сущности для поиска</param>
    ///     <param name="preference" type="BslLanguagePreference">Языковое предпочтение (Russian/English/Auto)</param>
    ///   </parameters>
    ///   <returns>Option<&BslEntity></returns>
    ///   <algorithm>
    ///     <step>Поиск по полному qualified_name (приоритетный)</step>
    ///     <step>Поиск по display_name</step>
    ///     <step>Поиск с учетом языковых предпочтений</step>
    ///   </algorithm>
    /// </api-method>
    pub fn find_entity_with_preference(&self, name: &str, preference: BslLanguagePreference) -> Option<&BslEntity> {
        // 1. Поиск по глобальным алиасам 1С (МАКСИМАЛЬНЫЙ приоритет)
        if let Some(id) = self.global_aliases.get(name) {
            return self.entities.get(id);
        }
        
        // 2. Поиск по полному qualified_name (высокий приоритет)
        if let Some(id) = self.by_qualified_name.get(name) {
            return self.entities.get(id);
        }
        
        // 3. Поиск по display_name (высокий приоритет)
        if let Some(id) = self.by_name.get(name) {
            return self.entities.get(id);
        }
        
        // 4. Оптимизированный поиск по языковым предпочтениям
        match preference {
            BslLanguagePreference::Russian => {
                // Сначала русские имена
                if let Some(id) = self.russian_names.get(name) {
                    return self.entities.get(id);
                }
                // Затем английские
                if let Some(id) = self.english_names.get(name) {
                    return self.entities.get(id);
                }
            }
            BslLanguagePreference::English => {
                // Сначала английские имена
                if let Some(id) = self.english_names.get(name) {
                    return self.entities.get(id);
                }
                // Затем русские
                if let Some(id) = self.russian_names.get(name) {
                    return self.entities.get(id);
                }
            }
            BslLanguagePreference::Auto => {
                // Определяем язык запроса и ищем соответственно
                if Self::contains_cyrillic(name) {
                    // Поиск кириллицы - сначала русские
                    if let Some(id) = self.russian_names.get(name) {
                        return self.entities.get(id);
                    }
                    if let Some(id) = self.english_names.get(name) {
                        return self.entities.get(id);
                    }
                } else {
                    // Поиск латиницы - сначала английские
                    if let Some(id) = self.english_names.get(name) {
                        return self.entities.get(id);
                    }
                    if let Some(id) = self.russian_names.get(name) {
                        return self.entities.get(id);
                    }
                }
            }
        }
        
        None
    }
    
    pub fn find_entity_by_id(&self, id: &BslEntityId) -> Option<&BslEntity> {
        self.entities.get(id)
    }
    
    pub fn find_types_with_method(&self, method_name: &str) -> Vec<&BslEntity> {
        self.methods_by_name.get(method_name)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.entities.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    pub fn find_types_with_property(&self, property_name: &str) -> Vec<&BslEntity> {
        self.properties_by_name.get(property_name)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.entities.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// <api-method>
    ///   <name>get_all_methods</name>
    ///   <purpose>Получение всех методов типа включая унаследованные</purpose>
    ///   <parameters>
    ///     <param name="entity_name" type="&str">Имя типа</param>
    ///   </parameters>
    ///   <returns>HashMap<String, BslMethod></returns>
    ///   <algorithm>
    ///     <step>Рекурсивный сбор методов от родительских типов</step>
    ///     <step>Добавление собственных методов (с переопределением)</step>
    ///   </algorithm>
    ///   <examples>
    ///     <example lang="rust">
    ///       // Получить все методы справочника
    ///       let methods = index.get_all_methods("Справочники.Номенклатура");
    ///       // Включает методы от СправочникОбъект, ОбъектБД и т.д.
    ///       
    ///       // Проверка конкретного метода
    ///       if methods.contains_key("Записать") {
    ///           println!("Объект можно записать");
    ///       }
    ///     </example>
    ///   </examples>
    /// </api-method>
    pub fn get_all_methods(&self, entity_name: &str) -> HashMap<String, BslMethod> {
        let mut all_methods = HashMap::new();
        
        if let Some(entity) = self.find_entity(entity_name) {
            // Собираем методы от родителей
            self.collect_inherited_methods(&entity.id, &mut all_methods);
            
            // Добавляем собственные методы (могут переопределять родительские)
            for (name, method) in &entity.interface.methods {
                all_methods.insert(name.clone(), method.clone());
            }
        }
        
        all_methods
    }
    
    fn collect_inherited_methods(&self, entity_id: &BslEntityId, methods: &mut HashMap<String, BslMethod>) {
        if let Some(&node_idx) = self.inheritance_node_map.get(entity_id) {
            // Обходим родителей
            for parent_idx in self.inheritance_graph.neighbors_directed(node_idx, Direction::Incoming) {
                if let Some(parent_id) = self.inheritance_graph.node_weight(parent_idx) {
                    // Рекурсивно собираем методы родителей
                    self.collect_inherited_methods(parent_id, methods);
                    
                    // Добавляем методы текущего родителя
                    if let Some(parent_entity) = self.entities.get(parent_id) {
                        for (name, method) in &parent_entity.interface.methods {
                            methods.insert(name.clone(), method.clone());
                        }
                    }
                }
            }
        }
    }
    
    pub fn get_all_properties(&self, entity_name: &str) -> HashMap<String, BslProperty> {
        let mut all_properties = HashMap::new();
        
        if let Some(entity) = self.find_entity(entity_name) {
            // Собираем свойства от родителей
            self.collect_inherited_properties(&entity.id, &mut all_properties);
            
            // Добавляем собственные свойства
            for (name, property) in &entity.interface.properties {
                all_properties.insert(name.clone(), property.clone());
            }
        }
        
        all_properties
    }
    
    fn collect_inherited_properties(&self, entity_id: &BslEntityId, properties: &mut HashMap<String, BslProperty>) {
        if let Some(&node_idx) = self.inheritance_node_map.get(entity_id) {
            for parent_idx in self.inheritance_graph.neighbors_directed(node_idx, Direction::Incoming) {
                if let Some(parent_id) = self.inheritance_graph.node_weight(parent_idx) {
                    self.collect_inherited_properties(parent_id, properties);
                    
                    if let Some(parent_entity) = self.entities.get(parent_id) {
                        for (name, property) in &parent_entity.interface.properties {
                            properties.insert(name.clone(), property.clone());
                        }
                    }
                }
            }
        }
    }
    
    /// <api-method>
    ///   <name>is_assignable</name>
    ///   <purpose>Проверка совместимости типов для присваивания</purpose>
    ///   <parameters>
    ///     <param name="from_type" type="&str">Исходный тип</param>
    ///     <param name="to_type" type="&str">Целевой тип</param>
    ///   </parameters>
    ///   <returns>bool - true если from_type можно присвоить to_type</returns>
    ///   <algorithm>
    ///     <step>Проверка точного совпадения типов</step>
    ///     <step>Проверка наследования через граф типов</step>
    ///     <step>Проверка интерфейсов (implements)</step>
    ///   </algorithm>
    ///   <examples>
    ///     <example lang="rust">
    ///       // Проверка совместимости справочника
    ///       let ok = index.is_assignable("Справочники.Номенклатура", "СправочникСсылка");
    ///       assert!(ok); // true - справочник реализует интерфейс СправочникСсылка
    ///       
    ///       // Проверка несовместимых типов
    ///       let ok = index.is_assignable("Число", "Строка");
    ///       assert!(!ok); // false - типы несовместимы
    ///     </example>
    ///   </examples>
    /// </api-method>
    pub fn is_assignable(&self, from_type: &str, to_type: &str) -> bool {
        if from_type == to_type {
            return true;
        }
        
        let from_entity = match self.find_entity(from_type) {
            Some(e) => e,
            None => return false,
        };
        
        // Проверяем прямое наследование
        if from_entity.constraints.parent_types.contains(&to_type.to_string()) {
            return true;
        }
        
        // Проверяем реализацию интерфейсов
        if from_entity.constraints.implements.contains(&to_type.to_string()) {
            return true;
        }
        
        // Проверяем транзитивное наследование через граф
        if let (Some(&from_node), Some(to_entity)) = (
            self.inheritance_node_map.get(&from_entity.id),
            self.find_entity(to_type)
        ) {
            if let Some(&to_node) = self.inheritance_node_map.get(&to_entity.id) {
                // Используем BFS для поиска пути
                use petgraph::algo::has_path_connecting;
                return has_path_connecting(&self.inheritance_graph, to_node, from_node, None);
            }
        }
        
        false
    }
    
    pub fn get_entity_count(&self) -> usize {
        self.entities.len()
    }
    
    pub fn get_entities_by_type(&self, entity_type: &BslEntityType) -> Vec<&BslEntity> {
        self.by_type.get(entity_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.entities.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    pub fn get_entities_by_kind(&self, entity_kind: &BslEntityKind) -> Vec<&BslEntity> {
        self.by_kind.get(entity_kind)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.entities.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    // Методы для ProjectIndexCache
    pub fn get_all_entities(&self) -> Vec<&BslEntity> {
        self.entities.values().collect()
    }
    
    pub fn get_by_name_index(&self) -> &HashMap<String, BslEntityId> {
        &self.by_name
    }
    
    pub fn get_by_qualified_name_index(&self) -> &HashMap<String, BslEntityId> {
        &self.by_qualified_name
    }
    
    /// Предлагает похожие имена для неточного поиска
    pub fn suggest_similar_names(&self, search_term: &str) -> Vec<String> {
        let search_lower = search_term.to_lowercase();
        let mut suggestions = Vec::new();
        
        // Ищем среди display_name
        for name in self.by_name.keys() {
            if name.to_lowercase().contains(&search_lower) {
                suggestions.push(name.clone());
            }
        }
        
        // Ищем среди qualified_name
        for name in self.by_qualified_name.keys() {
            if name.to_lowercase().contains(&search_lower) && !suggestions.contains(name) {
                suggestions.push(name.clone());
            }
        }
        
        // Ищем среди альтернативных имен
        for name in self.alternative_names.keys() {
            if name.to_lowercase().contains(&search_lower) {
                // Получаем полное имя из entity
                if let Some(entity_id) = self.alternative_names.get(name) {
                    if let Some(entity) = self.entities.get(entity_id) {
                        let full_name = &entity.display_name;
                        if !suggestions.contains(full_name) {
                            suggestions.push(full_name.clone());
                        }
                    }
                }
            }
        }
        
        // Ограничиваем до 10 предложений и сортируем
        suggestions.sort();
        suggestions.truncate(10);
        suggestions
    }
    
    /// ООП-подход: Автоматически наследует методы от соответствующего менеджера
    fn inherit_manager_methods(&self, mut entity: BslEntity) -> Result<BslEntity> {
        println!("🔍 Проверка наследования для: {} (вид: {:?})", entity.qualified_name, entity.entity_kind);
        
        // Мапинг типов конфигурации на их менеджеры (с правильными qualified_name)
        let manager_mappings = [
            (BslEntityKind::Catalog, "СправочникМенеджер.<Имя справочника> (CatalogManager.<Catalog name>)"),
            (BslEntityKind::Document, "ДокументМенеджер.<Имя документа> (DocumentManager.<Document name>)"),
            (BslEntityKind::InformationRegister, "РегистрСведенийМенеджер.<Имя регистра сведений> (InformationRegisterManager.<Information register name>)"),
            (BslEntityKind::AccumulationRegister, "РегистрНакопленияМенеджер.<Имя регистра накопления> (AccumulationRegisterManager.<Accumulation register name>)"),
            (BslEntityKind::DataProcessor, "ОбработкаМенеджер.<Имя обработки> (DataProcessorManager.<Data processor name>)"),
            (BslEntityKind::Report, "ОтчетМенеджер.<Имя отчета> (ReportManager.<Report name>)"),
        ];
        
        // Находим соответствующий менеджер для данного типа
        for (kind, manager_template) in &manager_mappings {
            if entity.entity_kind == *kind {
                println!("  ✅ Совпадение типа: {:?} → ищем шаблон {}", kind, manager_template);
                
                // Ищем шаблонный тип менеджера в платформенных типах
                if let Some(manager_entity) = self.entities.values()
                    .find(|e| e.qualified_name == *manager_template && e.entity_type == BslEntityType::Platform) {
                    
                    println!("🔄 Наследование методов: {} ← {}", entity.qualified_name, manager_template);
                    
                    // Копируем методы из менеджера в объект конфигурации
                    for (method_name, method) in &manager_entity.interface.methods {
                        if !entity.interface.methods.contains_key(method_name) {
                            entity.interface.methods.insert(method_name.clone(), method.clone());
                            println!("  ✅ Унаследован метод: {}", method_name);
                        }
                    }
                    
                    // Копируем свойства из менеджера
                    for (prop_name, prop) in &manager_entity.interface.properties {
                        if !entity.interface.properties.contains_key(prop_name) {
                            entity.interface.properties.insert(prop_name.clone(), prop.clone());
                            println!("  ✅ Унаследовано свойство: {}", prop_name);
                        }
                    }
                    
                    break; // Найден соответствующий менеджер
                } else {
                    println!("  ❌ Шаблонный тип {} не найден среди {} платформенных типов", 
                        manager_template, 
                        self.entities.values().filter(|e| e.entity_type == BslEntityType::Platform).count());
                    
                    // Отладка: ищем точный qualified_name
                    let exact_template = "СправочникМенеджер.<Имя справочника>";
                    if let Some(catalog_manager) = self.entities.values()
                        .find(|e| e.entity_type == BslEntityType::Platform && 
                                 e.qualified_name == exact_template) {
                        println!("    ✅ ТОЧНОЕ совпадение найдено!");
                        println!("       qualified_name: '{}'", catalog_manager.qualified_name);
                        println!("       методы: {}", catalog_manager.interface.methods.len());
                    } else {
                        // Покажем все типы, которые начинаются с "СправочникМенеджер.<Имя справочника>"
                        let related: Vec<_> = self.entities.values()
                            .filter(|e| e.entity_type == BslEntityType::Platform && 
                                       e.qualified_name.starts_with("СправочникМенеджер.<Имя справочника>"))
                            .map(|e| &e.qualified_name)
                            .take(5)
                            .collect();
                        println!("    ❌ Точное совпадение НЕ найдено");
                        println!("    📋 Связанные типы: {:?}", related);
                    }
                }
            }
        }
        
        Ok(entity)
    }
}