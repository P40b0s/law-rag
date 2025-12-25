use std::{collections::{BTreeMap, HashMap}, fmt::Debug};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use utilites::Date;
use crate::{Error, actual_redactions_client::DocumentResponse, models::Content};
const MAX_LVL: usize = 10;

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentNode<C: ToString + Debug>
{
    content_type: String,
    ///original http content
    original_content: String,
    /// converted via converter trait content
    converted_content: C,
    links: Option<Vec<String>>,
    content_start_id: usize,
    content_end_id: usize,
    content_lvl: usize,
    caption: String,
}
impl<C: ToString + Debug> DocumentNode<C> 
{
    pub fn new(
        content_type: &str,
        original_content: String,
        converted_content: C,
        links: Option<Vec<String>>,
        content_start_id: usize,
        content_end_id: usize,
        content_lvl: usize,
        caption: &str,
    ) -> Self 
    {
        Self 
        {
            content_type: content_type.to_string(),
            original_content,
            converted_content,
            links,
            content_start_id,
            content_end_id,
            content_lvl,
            caption: caption.to_string(),
        }
    }
    
    pub fn can_contain(&self, other: &DocumentNode<C>) -> bool 
    {
        other.content_start_id >= self.content_start_id &&
        other.content_end_id <= self.content_end_id &&
        other.content_lvl > self.content_lvl
    }
    pub fn content_type(&self) -> &str
    {
        &self.content_type
    }
    pub fn converted_content(&self) -> &C
    {
        &self.converted_content
    }
    pub fn original_content(&self) -> &str
    {
        &self.original_content
    }
    pub fn level(&self) -> usize
    {
        self.content_lvl
    }
    pub fn caption(&self) -> &str
    {
        &self.caption
    }
    pub fn links_hashes(&self) -> Option<&Vec<String>>
    {
        self.links.as_ref()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentNodes<C: ToString + Debug>
{
    hash: String,
    redaction_id: u32,
    name: String,
    number: String,
    sign_date: Date,
    publication_url: String,
    nodes: Vec<DocumentNode<C>>,
    //index -> (start, end)
    indexes: BTreeMap<usize, (usize, usize)>,
    // Дети для каждого узла
    children: HashMap<usize, Vec<usize>>,
    // Узлы по уровням
    by_level: [Vec<usize>; MAX_LVL], //максимальный уровень
}
impl<C: ToString + Debug> IntoIterator for DocumentNodes<C>
{
    type Item = DocumentNode<C>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter 
    {
        self.nodes.into_iter()
    }
}

impl<'a, C: ToString + Debug> IntoIterator for &'a DocumentNodes<C>
{
    type Item = &'a DocumentNode<C>;
    type IntoIter = std::slice::Iter<'a, DocumentNode<C>>;
    fn into_iter(self) -> Self::IntoIter 
    {
        self.nodes.iter()
    }
}
impl<C: ToString + Debug> Default for  DocumentNodes<C> 
{
    fn default() -> Self 
    {
        Self::new("default".to_owned(), "default".to_owned(), Date::now(), "default".to_owned(), "default".to_owned(), 0)
    }
}
impl<C: ToString + Debug> From<DocumentResponse> for DocumentNodes<C>
{
    fn from(value: DocumentResponse) -> Self 
    {
        Self::new(value.name, value.number, value.sign_date, value.publication_url, value.hash, value.redaction_id)
    }
}

impl<C: ToString + Debug> DocumentNodes<C> 
{
    pub fn new(name: String, number: String, sign_date: Date, publication_url: String, hash: String, redaction_id: u32) -> Self 
    {
        Self 
        {
            hash,
            redaction_id,
            name,
            number,
            sign_date,
            publication_url,
            nodes: Vec::with_capacity(2000),
            indexes: BTreeMap::new(),
            children: HashMap::with_capacity(2000),
            by_level: [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        }
    }

    fn find_parent_node(&self, node: &DocumentNode<C>) -> Option<&DocumentNode<C>>
    {
        if let Some(idx) = self.find_parent(node)
        {
            Some(&self.nodes[idx])
        }
        else 
        {
            None
        }
    }
    fn find_parent_node_by_range(&self, start: usize, end: usize, lvl: usize) -> Option<&DocumentNode<C>>
    {
        if let Some(idx) = self.find_parent_by_range(start, end, lvl)
        {
            Some(&self.nodes[idx])
        }
        else 
        {
            None
        }
    }
    fn find_parent(&self, node: &DocumentNode<C>) -> Option<usize>
    {
        if node.content_lvl == 0
        {
            None
        }
        else 
        {
            self.find_parent_by_range(node.content_start_id, node.content_end_id, node.content_lvl)
        }
    }
    fn find_parent_by_range(&self, start: usize, end: usize, lvl: usize) -> Option<usize>
    {
        if lvl == 0 { return None; }
        let parents_indexes = &self.by_level[lvl - 1];

        for parent_idx in parents_indexes
        {
            if let Some((exists_start, exists_end)) = self.indexes.get(parent_idx)
            {
                if *exists_start > start
                {
                    break;
                }
                if *exists_start <= start && *exists_end >= end
                {
                    return Some(*parent_idx);
                }
            }
        }
        
        None
    }
    pub fn insert(&mut self, node: DocumentNode<C>) -> Option<usize> 
    {
        let idx = self.nodes.len();
        let level = node.content_lvl;
        
        if level >= MAX_LVL
        {
            return None;
        }
        //если проверять на конфликты есть косяки...
        //было такое 135:165 статья 3
        //148:152 статтья 4
        //просто не проверяем конфликты а добавляем поочереди что есть
        // Проверяем конфликты на том же уровне
        for &existing_idx in &self.by_level[level] 
        {
            let existing = &self.nodes[existing_idx];
            if Self::ranges_overlap(
                existing.content_start_id, existing.content_end_id,
                node.content_start_id, node.content_end_id
            ) 
            {
                warn!("conflict! existing {}:{} with current {}:{}", existing.content_start_id, existing.content_end_id, node.content_start_id, node.content_end_id);
                //panic!("CONFLICT");
                //return None;
            }
        }

        self.indexes.insert(idx, (node.content_start_id, node.content_end_id));
        // Добавляем в индекс по уровням
        let pos = self.by_level[level]
            .binary_search_by(|&i| 
            {
                let (start, _) = self.indexes[&i];
                start.cmp(&node.content_start_id)
            })
            .unwrap_or_else(|pos| pos);
        
        self.by_level[level].insert(pos, idx);

        if level > 0 
        {
            if let Some(parent_idx) = self.find_parent(&node)
            {
                self.children.entry(parent_idx)
                .or_insert_with(Vec::new)
                .push(idx);
        
            }
        }
        // Сохраняем узел
        self.nodes.push(node);
        Some(idx)
    }
    
    fn ranges_overlap(start1: usize, end1: usize, start2: usize, end2: usize) -> bool 
    {
        !(end1 < start2 || end2 < start1)
    }
    
    fn has_conflict_with_siblings(&self, parent_idx: usize, new_node: &DocumentNode<C>) -> bool 
    {
        if let Some(siblings) = self.children.get(&parent_idx) 
        {
            for &sibling_idx in siblings 
            {
                let sibling = &self.nodes[sibling_idx];
                if Self::ranges_overlap(
                    sibling.content_start_id, sibling.content_end_id,
                    new_node.content_start_id, new_node.content_end_id
                ) 
                {
                    return true;
                }
            }
        }
        false
    }
    
    // Найти ВСЕХ родителей (все уровни)
    pub fn find_all_parents(&self, start: usize, end: usize, lvl: usize) -> Vec<&DocumentNode<C>> 
    {
        let mut result = Vec::new();
        let mut current = Some((start, end, lvl));
        while let Some((start, end, lvl)) = current && lvl > 0
        {
            if let Some(parent) = self.find_parent_node_by_range(start, end, lvl)
            {
                //info!("find parent {:?}", parent);
                current = Some((parent.content_start_id, parent.content_end_id, parent.content_lvl));
                result.push(parent);
            }
        }
        // Сортируем по уровню (от младшего к старшему)
        result.sort_by_key(|&node| node.content_lvl);
        result
    }
    pub fn find_all_parents_by_node(&self, node: &DocumentNode<C>) -> Vec<&DocumentNode<C>> 
    {
        self.find_all_parents(node.content_start_id, node.content_end_id, node.content_lvl)
    }
    pub fn find_all_parents_as_str(&self, node: &DocumentNode<C>) -> String 
    {
        let parents = self.find_all_parents(node.content_start_id, node.content_end_id, node.content_lvl);
        let mut p = parents
        .into_iter()
        .map(|p| p.caption.replace("$", "")).fold(String::new(), |acc, v|
        {
            if acc.is_empty()
            {
                v
            }
            else
            {
                acc + "->" + &v
            }
        });
        if p.is_empty()
        {
            p.push_str(&node.caption().replace("$", ""));
        }
        else 
        {
            p.push_str(&["->", &node.caption().replace("$", "")].concat());
        }
        p
        
    }
    
    pub fn get_children(&self, node_idx: usize) -> &[usize] {
        self.children.get(&node_idx).map(|v| v.as_slice()).unwrap_or(&[])
    }
    
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    
    pub fn print_tree(&self, node_idx: usize, depth: usize) 
    {
        let node = &self.nodes[node_idx];
        let indent = "  ".repeat(depth);
        println!("{}{}: {} [{}-{}] lvl:{}", 
            indent, node.content_type, node.caption,
            node.content_start_id, node.content_end_id, node.content_lvl);
        
        if let Some(children) = self.children.get(&node_idx) 
        {
            for &child_idx in children 
            {
                self.print_tree(child_idx, depth + 1);
            }
        }
    }

     pub fn stats(&self) -> StoreStats 
     {
        let mut stats = StoreStats::new();
        
        for (i, node) in self.nodes.iter().enumerate() 
        {
            stats.total_nodes += 1;
            stats.by_level[node.content_lvl as usize] += 1;
            
            if let Some(children) = self.children.get(&i) 
            {
                stats.nodes_with_children += 1;
                stats.total_children += children.len();
                stats.max_children = stats.max_children.max(children.len());
            }
        }
        
        stats
    }
    
    // Проверка целостности структуры
    pub fn validate(&self) -> ValidationResult 
    {
        let mut result = ValidationResult::new();
        
        // 1. Проверяем что все дети действительно внутри родителей
        for (parent_idx, children) in &self.children 
        {
            let parent = &self.nodes[*parent_idx];
            
            for &child_idx in children 
            {
                let child = &self.nodes[child_idx];
                
                if !parent.can_contain(child) {
                    result.errors.push(format!(
                        "Ребенок {}[{}] не содержится в родителе {}[{}]",
                        child.content_start_id, child.content_end_id,
                        parent.content_start_id, parent.content_end_id
                    ));
                }
                
                if child.content_lvl != parent.content_lvl + 1 
                {
                    result.errors.push(format!(
                        "Неверный уровень: родитель lvl={}, ребенок lvl={}",
                        parent.content_lvl, child.content_lvl
                    ));
                }
            }
        }
        
        // 2. Проверяем отсутствие пересечений на одном уровне
        for level in 0..MAX_LVL 
        {
            let indices = &self.by_level[level];
            
            for i in 0..indices.len() 
            {
                for j in (i + 1)..indices.len() 
                {
                    let node1 = &self.nodes[indices[i]];
                    let node2 = &self.nodes[indices[j]];
                    
                    if (node1.content_start_id >= node2.content_start_id && 
                         node1.content_start_id <= node2.content_end_id) ||
                       (node1.content_end_id >= node2.content_start_id && 
                         node1.content_end_id <= node2.content_end_id) {
                        result.warnings.push(format!(
                            "Пересечение на уровне {}: {}[{}] и {}[{}]",
                            level,
                            node1.content_start_id, node1.content_end_id,
                            node2.content_start_id, node2.content_end_id
                        ));
                    }
                }
            }
        }
        
        result.is_valid = result.errors.is_empty();
        result
    }
    pub fn redaction_id(&self) -> u32
    {
        self.redaction_id
    }
    pub fn hash(&self) -> &str
    {
        &self.hash
    }
    pub fn publication_url(&self) -> &str
    {
        &self.publication_url
    }
    pub fn number(&self) -> &str
    {
        &self.number
    }
    pub fn sign_date(&self) -> &Date
    {
        &self.sign_date
    }
    pub fn title(&self) -> &str
    {
        &self.name
    }
}


#[derive(Debug)]
pub struct StoreStats {
    pub total_nodes: usize,
    pub by_level: [usize; MAX_LVL],
    pub nodes_with_children: usize,
    pub total_children: usize,
    pub max_children: usize,
}

impl StoreStats {
    pub fn new() -> Self {
        Self {
            total_nodes: 0,
            by_level: [0; MAX_LVL],
            nodes_with_children: 0,
            total_children: 0,
            max_children: 0,
        }
    }
}

#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: false,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn print(&self) {
        if !self.errors.is_empty() {
            println!("ОШИБКИ:");
            for error in &self.errors {
                println!("  - {}", error);
            }
        }
        
        if !self.warnings.is_empty() {
            println!("ПРЕДУПРЕЖДЕНИЯ:");
            for warning in &self.warnings {
                println!("  - {}", warning);
            }
        }
        
        if self.errors.is_empty() && self.warnings.is_empty() {
            println!("✓ Все проверки пройдены успешно!");
        }
    }
}

#[cfg(test)]
mod tests
{
    use std::time::Instant;

    use rand::Rng;

    use crate::{document::{DocumentNode, DocumentNodes}, logger};

    // Тест 1: Базовый функционал
    #[test]
    fn test_basic_functionality() 
    {
        logger::init();
        println!("=== Тест 1: Базовый функционал ===");
        
        let mut store = DocumentNodes::default();
        
        // Создаем структуру:
        // L0: [0-1000] (корень)
        //   L1: [100-200]
        //     L2: [120-180]
        //       L3: [130-140]
        
        let root = DocumentNode::new("doc", "Root".to_string(),"Root".to_string(), None, 0, 1000, 0, "Root");
        let l1 = DocumentNode::new("section", "Section".to_string(),"Section".to_string(), None, 100, 200, 1, "Section 1");
        let l2 = DocumentNode::new("subsection", "Subsection".to_string(),"Subsection".to_string(), None, 120, 180, 2, "Subsection 1");
        let l3 = DocumentNode::new("paragraph", "Paragraph".to_string(),"Paragraph".to_string(), None, 130, 140, 3, "Paragraph 1");
        let root2 = DocumentNode::new("doc", "Root2".to_string(),"Root2".to_string(), None, 1001, 1100, 0, "Root2");
        
        assert_eq!(store.insert(root), Some(0));
        assert_eq!(store.insert(l1), Some(1));
        assert_eq!(store.insert(l2), Some(2));
        assert_eq!(store.insert(l3), Some(3));
        assert_eq!(store.insert(root2), Some(4));
        
        // Проверяем связи
        assert_eq!(store.get_children(0), &[1]);    // У корня есть ребенок L1
        assert_eq!(store.get_children(1), &[2]);    // У L1 есть ребенок L2
        assert_eq!(store.get_children(2), &[3]);    // У L2 есть ребенок L3
        assert_eq!(store.get_children(3), &[] as &[usize]);     // У L3 нет детей
        
        // Проверяем поиск родителя
        assert_eq!(store.find_parent_by_range(130, 140, 3), Some(2));
        assert_eq!(store.find_parent_by_range(120, 180, 2), Some(1));
        assert_eq!(store.find_parent_by_range(100, 200, 1), Some(0));
        assert_eq!(store.find_parent_by_range(0, 1000, 0), None);
        
        let stats = store.stats();
        println!("Статистика: {:?}", stats);
        
        let validation = store.validate();
        validation.print();
        
        println!("✓ Базовый функционал работает\n");
    }

    // Тест 2: Производительность вставки
    #[test]
    fn test_performance_insertion() {
        println!("=== Тест 2: Производительность вставки ===");
        
        let mut rng = rand::rng();
        let mut store = DocumentNodes::default();
        
        // Сначала добавляем корневые узлы уровня 1
        let mut level1_nodes = Vec::new();
        for i in 0..10 {
            let start = i * 100;
            let end = start + 99;
            level1_nodes.push(DocumentNode::new(
                "section",
                format!("Section {}", i),
                format!("Section {}", i),
                None,
                start,
                end,
                1,
                &format!("Section {}", i),
            ));
        }
        
        // Замеряем вставку уровня 1
        let start = Instant::now();
        for node in level1_nodes {
            store.insert(node);
        }
        let level1_time = start.elapsed();
        println!("Вставка 10 узлов уровня 1: {:?}", level1_time);
        
        // Вставляем узлы уровня 2
        let mut level2_nodes = Vec::new();
        for i in 0..100 {
            let parent_idx = i % 10;
            let parent_start = parent_idx * 100;
            let start = parent_start + rng.random_range(0..50);
            let end = start + rng.random_range(5..20);
            
            level2_nodes.push(DocumentNode::new(
                "subsection",
                format!("Subsection {}", i),
                format!("Subsection {}", i),
                None,
                start,
                end,
                2,
                &format!("Subsection {}", i),
            ));
        }
        
        let start = Instant::now();
        for node in level2_nodes {
            store.insert(node);
        }
        let level2_time = start.elapsed();
        println!("Вставка 100 узлов уровня 2: {:?}", level2_time);
        
        // Вставляем узлы уровня 3
        let mut level3_nodes = Vec::new();
        for i in 0..500 {
            let parent_idx = i % 100;
            // Для простоты теста используем фиксированные позиции
            let start = (i * 2) % 200 + 5;
            let end = start + 1;
            
            level3_nodes.push(DocumentNode::new(
                "paragraph",
                format!("Paragraph {}", i),
                format!("Paragraph {}", i),
                None,
                start,
                end,
                3,
                &format!("Paragraph {}", i),
            ));
        }
        
        let start = Instant::now();
        let mut successful = 0;
        for node in level3_nodes {
            if store.insert(node).is_some() {
                successful += 1;
            }
        }
        let level3_time = start.elapsed();
        println!("Вставка {} узлов уровня 3: {:?}", successful, level3_time);
        
        let stats = store.stats();
        println!("\nИтоговая статистика:");
        println!("Всего узлов: {}", stats.total_nodes);
        println!("По уровням: {:?}", stats.by_level);
        println!("Узлов с детьми: {}", stats.nodes_with_children);
        println!("Всего связей родитель-ребенок: {}", stats.total_children);
        println!("Максимальное число детей у узла: {}", stats.max_children);
        
        println!("\n✓ Тест производительности завершен\n");
    }

    // Тест 3: Конфликты интервалов
    #[test]
    fn test_interval_conflicts() {
        println!("=== Тест 3: Конфликты интервалов ===");
        
        let mut store = DocumentNodes::default();
        
        // Добавляем корневой узел
        store.insert(DocumentNode::new("doc", "Root".to_string(),"Root".to_string(),None, 0, 1000, 0, "Root"));
        
        // Узел уровня 1
        store.insert(DocumentNode::new("section", "S1".to_string(),"S1".to_string(),None, 100, 200, 1, "Section 1"));
        
        // Попытка добавить узел с пересекающимся интервалом на том же уровне
        let result = store.insert(DocumentNode::new("section", "S2".to_string(),"S2".to_string(),None, 150, 250, 1, "Section 2"));
        assert!(result.is_none(), "Должен быть отклонен из-за конфликта интервалов");
        
        // Добавляем узел без конфликта
        let result = store.insert(DocumentNode::new("section", "S3".to_string(),"S3".to_string(),None, 300, 400, 1, "Section 3"));
        assert!(result.is_some(), "Должен быть добавлен успешно");
        
        // Добавляем дочерний узел внутри первого
        let result = store.insert(DocumentNode::new("subsection", "SS1".to_string(),"SS1".to_string(),None, 110, 190, 2, "Subsection 1"));
        assert!(result.is_some(), "Должен быть добавлен как ребенок");
        
        // Попытка добавить узел уровня 2, который выходит за пределы родителя
        let result = store.insert(DocumentNode::new("subsection", "SS2".to_string(),"SS2".to_string(),None, 90, 210, 2, "Subsection 2"));
        assert!(result.is_none(), "Должен быть отклонен - выходит за пределы родителя");
        
        let validation = store.validate();
        println!("Результат валидации:");
        validation.print();
        
        println!("✓ Тест конфликтов завершен\n");
    }

    // Тест 4: Поиск родителей в сложной структуре
    #[test]
    fn test_parent_search_complex() {
        logger::init();
        println!("=== Тест 4: Поиск родителей в сложной структуре ===");
        
        let mut store = DocumentNodes::default();
        
        // Создаем вложенную структуру:
        // L1: [0-100], [200-300], [400-500]
        //   L2 внутри [0-100]: [10-30], [40-60], [70-90]
        //     L3 внутри [10-30]: [12-18], [20-25]
        
        store.insert(DocumentNode::new("doc", "Root".to_string(),"Root".to_string(),None, 0, 1000, 0, "Root"));
        
        // Уровень 1
        store.insert(DocumentNode::new("section", "S1".to_string(),"S1".to_string(),None, 0, 100, 1, "Section A"));
        store.insert(DocumentNode::new("section", "S2".to_string(),"S2".to_string(),None, 200, 300, 1, "Section B"));
        store.insert(DocumentNode::new("section", "S3".to_string(),"S3".to_string(),None, 400, 500, 1, "Section C"));
        
        // Уровень 2 внутри первого раздела
        store.insert(DocumentNode::new("subsection", "SS1".to_string(),"SS1".to_string(),None, 10, 30, 2, "Sub A1"));
        store.insert(DocumentNode::new("subsection", "SS2".to_string(),"SS2".to_string(),None, 40, 60, 2, "Sub A2"));
        store.insert(DocumentNode::new("subsection", "SS3".to_string(),"SS3".to_string(),None, 70, 90, 2, "Sub A3"));
        
        // Уровень 3
        store.insert(DocumentNode::new("paragraph", "P1".to_string(),"P1".to_string(),None, 12, 18, 3, "Para 1"));
        store.insert(DocumentNode::new("paragraph", "P2".to_string(),"P2".to_string(),None, 20, 25, 3, "Para 2"));
        
        // Тестируем поиск родителей
        println!("Поиск родителя для [12-18] lvl 3:");
        let parent = store.find_parent_by_range(12, 18, 3);
        assert!(parent.is_some());
        println!("  Найден родитель с idx: {:?}", parent);
        
        println!("\nПоиск родителя для [50-55] lvl 2:");
        let parent = store.find_parent_by_range(50, 55, 2);
        assert!(parent.is_some());
        println!("  Найден родитель с idx: {:?}", parent);
        
        println!("\nПоиск родителя для [250-280] lvl 2:");
        let parent = store.find_parent_by_range(250, 280, 2);
        assert!(parent.is_some());
        println!("  Найден родитель с idx: {:?}", parent);
        
        // Проверяем что у корня 3 ребенка
        assert_eq!(store.get_children(0).len(), 3);
        //проверяем всех родителей узла
        let all_parents = store.find_all_parents(20, 25, 3);
        assert_eq!(all_parents.len(), 3);

        let all_parents = store.find_all_parents(0, 1000, 0);
        assert_eq!(all_parents.len(), 0);
        // Проверяем что у первого раздела 3 ребенка уровня 2
        let first_section_idx = 1; // предполагаем что это первый добавленный
        let children = store.get_children(first_section_idx);
        println!("\nДети первого раздела: {} узлов", children.len());
        
        let stats = store.stats();
        println!("\nСтатистика дерева:");
        println!("Всего узлов: {}", stats.total_nodes);
        println!("Связей родитель-ребенок: {}", stats.total_children);
        
        println!("✓ Тест сложного поиска завершен\n");
    }

    // Тест 5: Загрузочный тест (нагрузочное тестирование)
    #[test]
    fn test_load_test() {
        println!("=== Тест 5: Нагрузочное тестирование ===");
        
        let mut store = DocumentNodes::default();
        let mut rng = rand::rng();
        
        // Добавляем корень
        store.insert(DocumentNode::new("doc", "Root".to_string(),"Root".to_string(),None, 0, 10000, 0, "Root"));
        
        let total_nodes = 2000;
        let mut inserted = 0;
        let mut conflicts = 0;
        
        let start_time = Instant::now();
        
        for i in 0..total_nodes {
            // Генерируем случайный уровень (1-3)
            let level = rng.random_range(1..=3);
            
            // Генерируем интервал в зависимости от уровня
            let size = match level {
                1 => rng.random_range(100..500),
                2 => rng.random_range(10..50),
                3 => rng.random_range(1..5),
                _ => 1,
            };
            
            let start = rng.random_range(0..(10000 - size));
            let end = start + size;
            
            let node = DocumentNode::new(
                "node",
                format!("Node {}", i),
                format!("Node {}", i),
                None,
                start,
                end,
                level,
                &format!("Node {}", i),
            );
            
            if store.insert(node).is_some() {
                inserted += 1;
            } else {
                conflicts += 1;
            }
            
            // Прогресс
            if i % 200 == 0 {
                let elapsed = start_time.elapsed();
                print!("\rПрогресс: {}/{} ({} конфликтов) [{:.2?}]", 
                    i, total_nodes, conflicts, elapsed);
            }
        }
        
        let total_time = start_time.elapsed();
        
        println!("\n\n=== РЕЗУЛЬТАТЫ НАГРУЗОЧНОГО ТЕСТА ===");
        println!("Всего попыток вставки: {}", total_nodes);
        println!("Успешно вставлено: {} ({:.1}%)", inserted, 
                inserted as f32 / total_nodes as f32 * 100.0);
        println!("Конфликтов: {} ({:.1}%)", conflicts,
                conflicts as f32 / total_nodes as f32 * 100.0);
        println!("Общее время: {:?}", total_time);
        println!("Среднее время на вставку: {:?}", 
                total_time / total_nodes as u32);
        
        let stats = store.stats();
        println!("\nСтатистика хранилища:");
        println!("Всего узлов: {}", stats.total_nodes);
        println!("Распределение по уровням:");
        for (level, count) in stats.by_level.iter().enumerate() {
            println!("  Уровень {}: {} узлов", level, count);
        }
        println!("Узлов с детьми: {}", stats.nodes_with_children);
        println!("Всего связей: {}", stats.total_children);
        println!("Макс детей у узла: {}", stats.max_children);
        
        // Быстрая валидация
        let validation = store.validate();
        println!("\nВалидация: {} ошибок, {} предупреждений", 
                validation.errors.len(), validation.warnings.len());
        
        println!("✓ Нагрузочный тест завершен\n");
    }

    // Тест 6: Сравнение производительности разных операций
    #[test]
    fn test_operations_benchmark() {
        println!("=== Тест 6: Бенчмарк операций ===");
        
        let mut store = DocumentNodes::default();
        let mut rng = rand::rng();
        
        // Подготовка: создаем тестовые данные
        let mut test_nodes = Vec::new();
        
        // 100 узлов уровня 1
        for i in 0..100 {
            let start = i * 10;
            test_nodes.push(DocumentNode::new(
                "section",
                format!("Section {}", i),
                format!("Section {}", i),
                None,
                start,
                start + 9,
                1,
                &format!("Section {}", i),
            ));
        }
        
        // 500 узлов уровня 2
        for i in 0..500 {
            let parent_start = (i % 10) * 10;
            let start = parent_start + rng.random_range(0..5);
            test_nodes.push(DocumentNode::new(
                "subsection",
                format!("Subsection {}", i),
                format!("Subsection {}", i),
                None,
                start,
                start + 2,
                2,
                &format!("Subsection {}", i),
            ));
        }
        
        // Замер 1: Вставка
        let start = Instant::now();
        for node in test_nodes {
            store.insert(node);
        }
        let insert_time = start.elapsed();
        println!("Вставка 600 узлов: {:?}", insert_time);
        
        // Замер 2: Поиск родителей (1000 поисков)
        let search_start = Instant::now();
        let mut found = 0;
        for _ in 0..1000 {
            let start = rng.random_range(0..100);
            let end = start + rng.random_range(1..5);
            let level = rng.random_range(1..=3);
            
            if store.find_parent_by_range(start, end, level).is_some() {
                found += 1;
            }
        }
        let search_time = search_start.elapsed();
        println!("1000 поисков родителей: {:?} (найдено: {})", search_time, found);
        
        // Замер 3: Получение детей (1000 запросов)
        let children_start = Instant::now();
        let mut total_children = 0;
        for _ in 0..1000 {
            let node_idx = rng.random_range(0..store.node_count());
            total_children += store.get_children(node_idx).len();
        }
        let children_time = children_start.elapsed();
        println!("1000 запросов детей: {:?} (всего детей: {})", children_time, total_children);
        
        println!("\nСреднее время на операцию:");
        println!("  Вставка: {:?}", insert_time / 600);
        println!("  Поиск родителя: {:?}", search_time / 1000);
        println!("  Получение детей: {:?}", children_time / 1000);
        
        println!("✓ Бенчмарк операций завершен\n");
    }
}
// {
// 	"data": [
// 		{
// 			"id": "a1",
// 			"np": "p6",
// 			"npe": "p117",
// 			"caption": "Статья 1",
// 			"unit": "статья",
// 			"lvl": 0
// 		},
// 		{
// 			"id": "a1_j1",
// 			"np": "p8",
// 			"npe": "p8",
// 			"caption": "$пункт 1",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a1_j2",
// 			"np": "p9",
// 			"npe": "p24",
// 			"caption": "$пункт 2",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a1_j2_k-1",
// 			"np": "p10",
// 			"npe": "p10",
// 			"caption": "$подпункт \"а\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j2_k-2",
// 			"np": "p11",
// 			"npe": "p14",
// 			"caption": "$подпункт \"б\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j2_k-3",
// 			"np": "p15",
// 			"npe": "p24",
// 			"caption": "$подпункт \"в\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j3",
// 			"np": "p25",
// 			"npe": "p32",
// 			"caption": "$пункт 3",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a1_j3_k-1",
// 			"np": "p26",
// 			"npe": "p26",
// 			"caption": "$подпункт \"а\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j3_k-2",
// 			"np": "p27",
// 			"npe": "p27",
// 			"caption": "$подпункт \"б\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j3_k-3",
// 			"np": "p28",
// 			"npe": "p28",
// 			"caption": "$подпункт \"в\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j3_k-4",
// 			"np": "p29",
// 			"npe": "p29",
// 			"caption": "$подпункт \"г\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j3_k-5",
// 			"np": "p30",
// 			"npe": "p31",
// 			"caption": "$подпункт \"д\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j3_k-6",
// 			"np": "p32",
// 			"npe": "p32",
// 			"caption": "$подпункт \"е\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j4",
// 			"np": "p33",
// 			"npe": "p39",
// 			"caption": "$пункт 4",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a1_j4_k-1",
// 			"np": "p34",
// 			"npe": "p36",
// 			"caption": "$подпункт \"а\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j4_k-2",
// 			"np": "p37",
// 			"npe": "p39",
// 			"caption": "$подпункт \"б\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j5",
// 			"np": "p40",
// 			"npe": "p46",
// 			"caption": "$пункт 5",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a1_j5_k-1",
// 			"np": "p41",
// 			"npe": "p44",
// 			"caption": "$подпункт \"а\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j5_k-2",
// 			"np": "p45",
// 			"npe": "p46",
// 			"caption": "$подпункт \"б\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j6",
// 			"np": "p47",
// 			"npe": "p51",
// 			"caption": "$пункт 6",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a1_j6_k-1",
// 			"np": "p48",
// 			"npe": "p48",
// 			"caption": "$подпункт \"а\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j6_k-2",
// 			"np": "p49",
// 			"npe": "p51",
// 			"caption": "$подпункт \"б\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j7",
// 			"np": "p52",
// 			"npe": "p65",
// 			"caption": "$пункт 7",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a1_j7_k-1",
// 			"np": "p53",
// 			"npe": "p58",
// 			"caption": "$подпункт \"а\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j7_k-2",
// 			"np": "p59",
// 			"npe": "p59",
// 			"caption": "$подпункт \"б\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j7_k-3",
// 			"np": "p60",
// 			"npe": "p64",
// 			"caption": "$подпункт \"в\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j7_k-4",
// 			"np": "p65",
// 			"npe": "p65",
// 			"caption": "$подпункт \"г\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j8",
// 			"np": "p66",
// 			"npe": "p101",
// 			"caption": "$пункт 8",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a1_j9",
// 			"np": "p102",
// 			"npe": "p102",
// 			"caption": "$пункт 9",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a1_j10",
// 			"np": "p103",
// 			"npe": "p104",
// 			"caption": "$пункт 10",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a1_j11",
// 			"np": "p105",
// 			"npe": "p108",
// 			"caption": "$пункт 11",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a1_j11_k-1",
// 			"np": "p106",
// 			"npe": "p106",
// 			"caption": "$подпункт \"а\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j11_k-2",
// 			"np": "p107",
// 			"npe": "p107",
// 			"caption": "$подпункт \"б\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j11_k-3",
// 			"np": "p108",
// 			"npe": "p108",
// 			"caption": "$подпункт \"в\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j12",
// 			"np": "p109",
// 			"npe": "p109",
// 			"caption": "$пункт 12",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a1_j13",
// 			"np": "p110",
// 			"npe": "p110",
// 			"caption": "$пункт 13",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a1_j14",
// 			"np": "p111",
// 			"npe": "p117",
// 			"caption": "$пункт 14",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a1_j14_k-1",
// 			"np": "p112",
// 			"npe": "p112",
// 			"caption": "$подпункт \"а\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j14_k-2",
// 			"np": "p113",
// 			"npe": "p113",
// 			"caption": "$подпункт \"б\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a1_j14_k-3",
// 			"np": "p114",
// 			"npe": "p117",
// 			"caption": "$подпункт \"в\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a2",
// 			"np": "p118",
// 			"npe": "p134",
// 			"caption": "Статья 2",
// 			"unit": "статья",
// 			"lvl": 0
// 		},
// 		{
// 			"id": "a2_j1",
// 			"np": "p120",
// 			"npe": "p122",
// 			"caption": "$пункт 1",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a2_j1_k-1",
// 			"np": "p121",
// 			"npe": "p121",
// 			"caption": "$подпункт \"а\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a2_j1_k-2",
// 			"np": "p122",
// 			"npe": "p122",
// 			"caption": "$подпункт \"б\"",
// 			"unit": "подпункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a2_j2",
// 			"np": "p123",
// 			"npe": "p123",
// 			"caption": "$пункт 2",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a2_j3",
// 			"np": "p124",
// 			"npe": "p124",
// 			"caption": "$пункт 3",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a2_j4",
// 			"np": "p125",
// 			"npe": "p134",
// 			"caption": "$пункт 4",
// 			"unit": "пункт",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a3",
// 			"np": "p135",
// 			"npe": "p165",
// 			"caption": "Статья 3",
// 			"unit": "статья",
// 			"lvl": 0
// 		},
// 		{
// 			"id": "a3_c1",
// 			"np": "p136",
// 			"npe": "p136",
// 			"caption": "$часть 1",
// 			"unit": "часть",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a3_c2",
// 			"np": "p137",
// 			"npe": "p137",
// 			"caption": "$часть 2",
// 			"unit": "часть",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a3_c3",
// 			"np": "p138",
// 			"npe": "p138",
// 			"caption": "$часть 3",
// 			"unit": "часть",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a3_c4",
// 			"np": "p139",
// 			"npe": "p139",
// 			"caption": "$часть 4",
// 			"unit": "часть",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a3_c5",
// 			"np": "p140",
// 			"npe": "p140",
// 			"caption": "$часть 5",
// 			"unit": "часть",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a3_c6",
// 			"np": "p141",
// 			"npe": "p141",
// 			"caption": "$часть 6",
// 			"unit": "часть",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a3_c7",
// 			"np": "p142",
// 			"npe": "p142",
// 			"caption": "$часть 7",
// 			"unit": "часть",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a3_c8",
// 			"np": "p143",
// 			"npe": "p146",
// 			"caption": "$часть 8",
// 			"unit": "часть",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a3_c8_j1",
// 			"np": "p144",
// 			"npe": "p144",
// 			"caption": "$пункт 1",
// 			"unit": "пункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a3_c8_j2",
// 			"np": "p145",
// 			"npe": "p145",
// 			"caption": "$пункт 2",
// 			"unit": "пункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a3_c8_j3",
// 			"np": "p146",
// 			"npe": "p146",
// 			"caption": "$пункт 3",
// 			"unit": "пункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a3_c9",
// 			"np": "p147",
// 			"npe": "p147",
// 			"caption": "$часть 9",
// 			"unit": "часть",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a3_c10",
// 			"np": "p157",
// 			"npe": "p163",
// 			"caption": "$часть 10",
// 			"unit": "часть",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a3_c10_j2",
// 			"np": "p159",
// 			"npe": "p162",
// 			"caption": "$пункт 2",
// 			"unit": "пункт",
// 			"lvl": 2
// 		},
// 		{
// 			"id": "a3_c10_j2_k-1",
// 			"np": "p160",
// 			"npe": "p160",
// 			"caption": "$подпункт \"а\"",
// 			"unit": "подпункт",
// 			"lvl": 3
// 		},
// 		{
// 			"id": "a3_c10_j2_k-2",
// 			"np": "p161",
// 			"npe": "p161",
// 			"caption": "$подпункт \"б\"",
// 			"unit": "подпункт",
// 			"lvl": 3
// 		},
// 		{
// 			"id": "a3_c10_j2_k-3",
// 			"np": "p162",
// 			"npe": "p162",
// 			"caption": "$подпункт \"в\"",
// 			"unit": "подпункт",
// 			"lvl": 3
// 		},
// 		{
// 			"id": "a3_c12",
// 			"np": "p165",
// 			"npe": "p165",
// 			"caption": "$часть 12",
// 			"unit": "часть",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a4",
// 			"np": "p148",
// 			"npe": "p152",
// 			"caption": "Статья 4",
// 			"unit": "статья",
// 			"lvl": 0
// 		},
// 		{
// 			"id": "a4_c1",
// 			"np": "p149",
// 			"npe": "p149",
// 			"caption": "$часть 1",
// 			"unit": "часть",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a4_c2",
// 			"np": "p150",
// 			"npe": "p150",
// 			"caption": "$часть 2",
// 			"unit": "часть",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a4_c3",
// 			"np": "p151",
// 			"npe": "p151",
// 			"caption": "$часть 3",
// 			"unit": "часть",
// 			"lvl": 1
// 		},
// 		{
// 			"id": "a4_c4",
// 			"np": "p152",
// 			"npe": "p152",
// 			"caption": "$часть 4",
// 			"unit": "часть",
// 			"lvl": 1
// 		}
// 	],
// 	"complete": true,
// 	"error": null,
// 	"status": 0,
// 	"typeact": "федеральный закон",
// 	"lockkey": 0
// }