use std::collections::HashMap;

use rangemap::RangeInclusiveMap;
use scraper::Selector;
use utilites::Date;

use rag_core::Converter;

use crate::actual_redactions_client::DocumentResponse;
#[derive(Debug, Clone)]
pub struct TreeNode
{
    pub id: String,
    pub content_type: String,
    pub caption: String,
    /// Конкатенация HTML-абзацев, принадлежащих только этому узлу
    /// (абзацы дочерних узлов не включаются — они находятся в своих TreeNode).
    pub text: String,
    pub links: Option<Vec<String>>,
    pub para_start: usize,
    pub para_end: usize,
    pub lvl: usize,
}

/// Дерево документа, построенное из `Contents` + HTML.
///
/// Ключевые решения:
/// - Иерархия строится по `lvl` + `RangeInclusiveMap`.
/// - `RangeInclusiveMap` (rangemap) вставляется в порядке возрастания `lvl`:
///   более специфичный дочерний диапазон «вырезает» себя из родительского —
///   после всех вставок `.get(para_num)` за O(log n) даёт самый глубокий узел.
/// - Родитель каждого узла ищется в той же карте до его собственной вставки
///   (в момент поиска там лежат только узлы с `lvl < current`).
#[derive(Debug, Clone)]
pub struct DocumentTree
{
    pub hash: String,
    pub redaction_id: u32,
    pub name: String,
    pub number: String,
    pub sign_date: Date,
    pub publication_url: String,
    nodes: Vec<TreeNode>,
    idx_by_id: HashMap<String, usize>,
    /// child_id -> parent_id
    parent_of: HashMap<String, String>,
    /// parent_id -> [child_id, ...]
    children: HashMap<String, Vec<String>>,
}

impl DocumentTree
{
    pub fn build<CONV: Converter<String>>(response: DocumentResponse, converter: CONV) -> Self
    {
        let DocumentResponse { html, contents, name, number, sign_date, publication_url, hash, redaction_id } = response;

        // Парсим диапазоны; отбрасываем записи с невалидными полями np/npe.
        struct Item { id: String, unit: String, caption: String, lvl: usize, start: usize, end: usize }
        let mut items: Vec<Item> = contents.content
            .into_iter()
            .filter_map(|c|
            {
                let start = c.paragraph_start_number.strip_prefix("p")?.parse().ok()?;
                let end   = c.paragraph_end_number.strip_prefix("p")?.parse().ok()?;
                Some(Item { id: c.id, unit: c.unit, caption: c.caption, lvl: c.lvl, start, end })
            })
            .collect();

        // Сортируем по lvl ASC.
        items.sort_by_key(|i| i.lvl);

        let max_lvl = items.iter().map(|i| i.lvl).max().unwrap_or(0);

        // Отдельная карта на каждый уровень — для поиска родителя.
        // Родитель узла на lvl=N ищется ТОЛЬКО в карте lvl=N-1,
        // чтобы узлы-сиблинги (одинаковый lvl, но вложенные диапазоны)
        // не считались родителями.
        let mut level_maps: Vec<RangeInclusiveMap<usize, String>> =
            (0..=max_lvl).map(|_| RangeInclusiveMap::new()).collect();

        // Глобальная карта для назначения абзацев: вставляем в порядке
        // возрастания lvl, последний вставленный побеждает → .get(para) даёт
        // самый глубокий (специфичный) узел.
        let mut para_map: RangeInclusiveMap<usize, String> = RangeInclusiveMap::new();

        let mut parent_of: HashMap<String, String> = HashMap::new();
        let mut children:  HashMap<String, Vec<String>> = HashMap::new();

        for item in &items
        {
            // Поиск родителя — только в карте предыдущего уровня.
            if item.lvl > 0
            {
                if let Some(parent_id) = level_maps[item.lvl - 1].get(&item.start)
                {
                    parent_of.insert(item.id.clone(), parent_id.clone());
                    children.entry(parent_id.clone()).or_default().push(item.id.clone());
                }
            }
            level_maps[item.lvl].insert(item.start..=item.end, item.id.clone());
            para_map.insert(item.start..=item.end, item.id.clone());
        }

        // Разбираем HTML и назначаем текст/ссылки узлам.
        let para_sel  = Selector::parse("p:not(.I):not(.C):not(.T):not(.Z):not(.Y):not(.mark):not(.markx)").unwrap();
        let links_sel = Selector::parse("span[cmdprm]").unwrap();

        let mut node_texts: HashMap<String, Vec<(String, Vec<String>)>> = HashMap::new();

        for p in html.select(&para_sel)
        {
            let para_num: usize = match p.attr("id")
                .and_then(|id| id.strip_prefix("p"))
                .and_then(|n| n.parse().ok())
            {
                Some(n) => n,
                None    => continue,
            };

            // O(log n)
            let owner_id = match para_map.get(&para_num)
            {
                Some(id) => id.clone(),
                None     => continue,
            };

            let links: Vec<String> = p.select(&links_sel)
                .filter_map(|l|
                {
                    l.attr("cmdprm").and_then(|cmd|
                        cmd.split_once(' ').map(|(left, _)| left.replace("gohash=", ""))
                    )
                })
                .collect();

            node_texts.entry(owner_id).or_default().push((converter.convert(p.html()), links));
        }

        // Собираем TreeNode, сортируем в порядке документа.
        let mut nodes: Vec<TreeNode> = items
            .into_iter()
            .map(|item|
            {
                let (text, links) = match node_texts.remove(&item.id)
                {
                    Some(paras) =>
                    {
                        let text = paras.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>().join("\n");
                        let all_links: Vec<String> = paras.into_iter().flat_map(|(_, l)| l).collect();
                        (text, if all_links.is_empty() { None } else { Some(all_links) })
                    }
                    None => (String::new(), None),
                };
                TreeNode
                {
                    id: item.id, content_type: item.unit, caption: item.caption,
                    text, links, para_start: item.start, para_end: item.end, lvl: item.lvl,
                }
            })
            .collect();

        nodes.sort_by_key(|n| n.para_start);

        let idx_by_id: HashMap<String, usize> = nodes
            .iter().enumerate().map(|(i, n)| (n.id.clone(), i)).collect();

        DocumentTree { hash, redaction_id, name, number, sign_date, publication_url, nodes, idx_by_id, parent_of, children }
    }

    pub fn iter(&self) -> impl Iterator<Item = &TreeNode>
    {
        self.nodes.iter()
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }

    /// Предки узла от корня к непосредственному родителю.
    /// Использует `parent_of` (ключ — id, построен по `lvl`) — без разбора строк id.
    pub fn ancestors(&self, node: &TreeNode) -> Vec<&TreeNode>
    {
        let mut result = Vec::new();
        let mut current_id = node.id.as_str();
        while let Some(parent_id) = self.parent_of.get(current_id)
        {
            if let Some(&idx) = self.idx_by_id.get(parent_id.as_str())
            {
                result.push(&self.nodes[idx]);
                current_id = &self.nodes[idx].id;
            }
            else { break; }
        }
        result.reverse();
        result
    }

    /// "статья 1->часть 8->пункт 2"
    pub fn path_str(&self, node: &TreeNode) -> String
    {
        
        let mut parts: Vec<String> = self.ancestors(node)
            .iter()
            .map(|n| part_parser(&n.caption))
            .collect();
        parts.push(part_parser(&node.caption));
        parts.join("->")
    }

    pub fn children_of(&self, id: &str) -> &[String]
    {
        self.children.get(id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn publication_url(&self) -> &str { &self.publication_url }
    pub fn hash(&self)            -> &str { &self.hash }
    pub fn title(&self)           -> &str { &self.name }
    pub fn number(&self)          -> &str { &self.number }
    pub fn sign_date(&self)       -> &Date { &self.sign_date }
}

/// "$часть 1"
/// "Статья 1. Общие положения
fn part_parser(part: &str) -> String
{
    if let Some((name, number)) = part.split_once(" ")
    {
        let name = name.replace("$", "").to_lowercase();
        let number = number.chars().take_while(|w| w.is_digit(10)).collect();
        [name, " ".to_owned(), number].concat()
    }
    else 
    {
        part.to_owned()
    }
}