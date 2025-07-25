/*!
# LRU Cache Implementation

Быстрая реализация LRU (Least Recently Used) кэша для использования
в анализаторе BSL. Обеспечивает O(1) операции get/put с автоматическим
вытеснением наименее используемых элементов.
*/

use std::collections::HashMap;
use std::hash::Hash;
use std::ptr::NonNull;
use serde::{Deserialize, Serialize};

/// LRU кэш с заданной емкостью
pub struct LruCache<K: Clone + Eq + Hash, V: Clone> {
    /// Хэш-таблица для быстрого доступа к узлам
    map: HashMap<K, NonNull<Node<K, V>>>,
    /// Указатель на голову списка (самый новый)
    head: Option<NonNull<Node<K, V>>>,
    /// Указатель на хвост списка (самый старый)
    tail: Option<NonNull<Node<K, V>>>,
    /// Максимальная емкость кэша
    capacity: usize,
    /// Статистика кэша
    stats: CacheStats,
}

/// Узел двусвязного списка
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<NonNull<Node<K, V>>>,
    next: Option<NonNull<Node<K, V>>>,
}

/// Статистика кэша
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Общее количество операций get
    pub get_operations: u64,
    /// Количество попаданий
    pub hits: u64,
    /// Количество промахов
    pub misses: u64,
    /// Количество операций put
    pub put_operations: u64,
    /// Количество вытеснений
    pub evictions: u64,
}

impl<K, V> Node<K, V> {
    fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            prev: None,
            next: None,
        }
    }
}

impl<K: Clone + Eq + Hash, V: Clone> LruCache<K, V> {
    /// Создает новый LRU кэш с заданной емкостью
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            head: None,
            tail: None,
            capacity,
            stats: CacheStats::default(),
        }
    }
    
    /// Получает значение по ключу
    pub fn get(&mut self, key: &K) -> Option<V> {
        self.stats.get_operations += 1;
        
        if let Some(&node_ptr) = self.map.get(key) {
            // Перемещаем узел в начало списка
            unsafe {
                let node = node_ptr.as_ref();
                let value = node.value.clone();
                self.move_to_head(node_ptr);
                self.stats.hits += 1;
                Some(value)
            }
        } else {
            self.stats.misses += 1;
            None
        }
    }
    
    /// Вставляет или обновляет значение
    pub fn put(&mut self, key: K, value: V) {
        self.stats.put_operations += 1;
        
        let existing_node = self.map.get(&key).copied();
        if let Some(mut existing_node) = existing_node {
            // Обновляем существующий узел
            unsafe {
                let node = existing_node.as_mut();
                node.value = value;
                self.move_to_head(existing_node);
            }
        } else {
            // Создаем новый узел
            let new_node = Box::leak(Box::new(Node::new(key.clone(), value)));
            let new_node_ptr = NonNull::from(new_node);
            
            unsafe {
                self.add_to_head(new_node_ptr);
            }
            self.map.insert(key, new_node_ptr);
            
            // Проверяем емкость и вытесняем если нужно
            if self.map.len() > self.capacity {
                self.remove_tail();
                self.stats.evictions += 1;
            }
        }
    }
    
    /// Удаляет элемент по ключу
    pub fn pop(&mut self, key: &K) -> Option<V> {
        if let Some(&node_ptr) = self.map.get(key) {
            unsafe {
                let node = Box::from_raw(node_ptr.as_ptr());
                self.remove_node(node_ptr);
                self.map.remove(key);
                Some(node.value)
            }
        } else {
            None
        }
    }
    
    /// Удаляет и возвращает наименее используемый элемент
    pub fn pop_lru(&mut self) -> Option<(K, V)> {
        if let Some(tail_ptr) = self.tail {
            unsafe {
                let tail_node = Box::from_raw(tail_ptr.as_ptr());
                let key = tail_node.key.clone();
                let value = tail_node.value.clone();
                
                self.remove_node(tail_ptr);
                self.map.remove(&key);
                
                Some((key, value))
            }
        } else {
            None
        }
    }
    
    /// Проверяет, содержится ли ключ в кэше
    pub fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }
    
    /// Возвращает количество элементов в кэше
    pub fn len(&self) -> usize {
        self.map.len()
    }
    
    /// Проверяет, пуст ли кэш
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    
    /// Возвращает емкость кэша
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Очищает кэш
    pub fn clear(&mut self) {
        // Освобождаем все узлы
        while let Some((_, _)) = self.pop_lru() {
            // Узлы освобождаются в pop_lru
        }
        
        self.map.clear();
        self.head = None;
        self.tail = None;
    }
    
    /// Возвращает статистику кэша
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }
    
    /// Сбрасывает статистику
    pub fn reset_stats(&mut self) {
        self.stats = CacheStats::default();
    }
    
    /// Возвращает итератор по элементам кэша
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter().map(|(k, &node_ptr)| {
            unsafe {
                let node = node_ptr.as_ref();
                (k, &node.value)
            }
        })
    }
    
    /// Добавляет узел в начало списка
    unsafe fn add_to_head(&mut self, mut node_ptr: NonNull<Node<K, V>>) {
        let node = node_ptr.as_mut();
        
        node.prev = None;
        node.next = self.head;
        
        if let Some(mut head_ptr) = self.head {
            head_ptr.as_mut().prev = Some(node_ptr);
        } else {
            // Первый элемент - становится и головой и хвостом
            self.tail = Some(node_ptr);
        }
        
        self.head = Some(node_ptr);
    }
    
    /// Перемещает узел в начало списка
    unsafe fn move_to_head(&mut self, node_ptr: NonNull<Node<K, V>>) {
        if self.head == Some(node_ptr) {
            // Узел уже в начале
            return;
        }
        
        self.remove_node(node_ptr);
        self.add_to_head(node_ptr);
    }
    
    /// Удаляет узел из списка
    unsafe fn remove_node(&mut self, node_ptr: NonNull<Node<K, V>>) {
        let node = node_ptr.as_ref();
        
        // Обновляем связи соседних узлов
        if let Some(mut prev_ptr) = node.prev {
            prev_ptr.as_mut().next = node.next;
        } else {
            // Это была голова
            self.head = node.next;
        }
        
        if let Some(mut next_ptr) = node.next {
            next_ptr.as_mut().prev = node.prev;
        } else {
            // Это был хвост
            self.tail = node.prev;
        }
    }
    
    /// Удаляет последний элемент (хвост)
    fn remove_tail(&mut self) {
        if let Some(tail_ptr) = self.tail {
            unsafe {
                let tail_node = Box::from_raw(tail_ptr.as_ptr());
                self.remove_node(tail_ptr);
                self.map.remove(&tail_node.key);
            }
        }
    }
}

impl<K: Clone + Eq + Hash, V: Clone> Drop for LruCache<K, V> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl CacheStats {
    /// Возвращает коэффициент попаданий (0.0 - 1.0)
    pub fn hit_rate(&self) -> f64 {
        if self.get_operations == 0 {
            0.0
        } else {
            self.hits as f64 / self.get_operations as f64
        }
    }
    
    /// Возвращает коэффициент промахов (0.0 - 1.0)
    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate()
    }
    
    /// Возвращает эффективность кэша (попадания / (попадания + промахи))
    pub fn efficiency(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            self.hits as f64 / (self.hits + self.misses) as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lru_cache_basic_operations() {
        let mut cache = LruCache::new(2);
        
        // Вставляем элементы
        cache.put(1, "one");
        cache.put(2, "two");
        
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&1), Some("one"));
        assert_eq!(cache.get(&2), Some("two"));
    }
    
    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = LruCache::new(2);
        
        cache.put(1, "one");
        cache.put(2, "two");
        cache.put(3, "three"); // Должно вытеснить 1
        
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some("two"));
        assert_eq!(cache.get(&3), Some("three"));
    }
    
    #[test]
    fn test_lru_cache_update_existing() {
        let mut cache = LruCache::new(2);
        
        cache.put(1, "one");
        cache.put(2, "two");
        cache.put(1, "ONE"); // Обновляем существующий
        cache.put(3, "three"); // Должно вытеснить 2
        
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&1), Some("ONE"));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some("three"));
    }
    
    #[test]
    fn test_lru_cache_access_order() {
        let mut cache = LruCache::new(3);
        
        cache.put(1, "one");
        cache.put(2, "two");
        cache.put(3, "three");
        
        // Получаем 1, делая его самым новым
        cache.get(&1);
        
        cache.put(4, "four"); // Должно вытеснить 2
        
        assert_eq!(cache.len(), 3);
        assert_eq!(cache.get(&1), Some("one"));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some("three"));
        assert_eq!(cache.get(&4), Some("four"));
    }
    
    #[test]
    fn test_lru_cache_pop() {
        let mut cache = LruCache::new(3);
        
        cache.put(1, "one");
        cache.put(2, "two");
        cache.put(3, "three");
        
        assert_eq!(cache.pop(&2), Some("two"));
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&2), None);
    }
    
    #[test]
    fn test_lru_cache_pop_lru() {
        let mut cache = LruCache::new(3);
        
        cache.put(1, "one");
        cache.put(2, "two");
        cache.put(3, "three");
        
        // 1 должен быть наименее используемым
        assert_eq!(cache.pop_lru(), Some((1, "one")));
        assert_eq!(cache.len(), 2);
    }
    
    #[test]
    fn test_lru_cache_clear() {
        let mut cache = LruCache::new(3);
        
        cache.put(1, "one");
        cache.put(2, "two");
        cache.put(3, "three");
        
        cache.clear();
        
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert_eq!(cache.get(&1), None);
    }
    
    #[test]
    fn test_lru_cache_stats() {
        let mut cache = LruCache::new(2);
        
        cache.put(1, "one");
        cache.put(2, "two");
        
        cache.get(&1); // hit
        cache.get(&3); // miss
        cache.get(&2); // hit
        
        let stats = cache.stats();
        assert_eq!(stats.put_operations, 2);
        assert_eq!(stats.get_operations, 3);
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 2.0 / 3.0);
    }
    
    #[test]
    fn test_lru_cache_contains() {
        let mut cache = LruCache::new(2);
        
        cache.put(1, "one");
        
        assert!(cache.contains(&1));
        assert!(!cache.contains(&2));
    }
    
    #[test]
    fn test_lru_cache_iterator() {
        let mut cache = LruCache::new(3);
        
        cache.put(1, "one");
        cache.put(2, "two");
        cache.put(3, "three");
        
        let items: Vec<_> = cache.iter().collect();
        assert_eq!(items.len(), 3);
        
        // Проверяем, что все элементы присутствуют
        assert!(items.iter().any(|(&k, &v)| k == 1 && v == "one"));
        assert!(items.iter().any(|(&k, &v)| k == 2 && v == "two"));
        assert!(items.iter().any(|(&k, &v)| k == 3 && v == "three"));
    }
}