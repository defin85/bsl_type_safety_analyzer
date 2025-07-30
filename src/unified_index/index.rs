use std::collections::HashMap;
use anyhow::Result;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use serde::{Serialize, Deserialize};

use super::entity::{BslEntity, BslEntityId, BslEntityType, BslEntityKind, BslMethod, BslProperty};

#[derive(Serialize, Deserialize)]
pub struct UnifiedBslIndex {
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
        Self {
            entities: HashMap::new(),
            by_name: HashMap::new(),
            by_qualified_name: HashMap::new(),
            by_type: HashMap::new(),
            by_kind: HashMap::new(),
            methods_by_name: HashMap::new(),
            properties_by_name: HashMap::new(),
            inheritance_graph: DiGraph::new(),
            inheritance_node_map: HashMap::new(),
            reference_graph: DiGraph::new(),
            reference_node_map: HashMap::new(),
        }
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
        self.by_name.insert(name, id.clone());
        self.by_qualified_name.insert(qualified_name, id.clone());
        
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
        
        // Сохраняем сущность
        self.entities.insert(id, entity);
        
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
    
    pub fn find_entity(&self, name: &str) -> Option<&BslEntity> {
        self.by_qualified_name.get(name)
            .or_else(|| self.by_name.get(name))
            .and_then(|id| self.entities.get(id))
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
}