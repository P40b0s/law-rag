mod ibpi_client;
mod error;
mod actual_redactions_client;
mod encoding;
mod models;
mod search_attributes;
mod document;
mod html_to_markdown;
mod converter;
pub use error::Error;
mod logger;
mod parser;
use crate::{actual_redactions_client::{ActualRedactionsClient}, error::Result, models::ContentItem};
use std::{collections::BTreeMap, fmt::Debug};
use scraper::{Selector};
use tracing::{debug, info};
use utilites::Date;
pub use document::{DocumentNode, DocumentNodes};
pub use converter::Converter;

pub struct SystemaClient
{
}
impl SystemaClient
{
    ///Date::new_date(29, 05, 2024), "102-ФЗ"
    pub async fn get_document<CONV, CONT>(sign_date: Date, number: &str, converter: CONV) -> Result<DocumentNodes<CONT>>
    where   CONT: ToString + Debug,
            CONV: converter::Converter<CONT>

    {
        let document = ActualRedactionsClient::get_document(sign_date, number).await?;
        let contents = document.contents;
        let html = document.html;
        let mut content_map = BTreeMap::new();
        let mut document_nodes = DocumentNodes::new(document.name, document.number, document.sign_date, document.publication_url, document.hash, document.redaction_id);
        for content in contents.content
        {
            let item: Result<ContentItem> = content.try_into();
            if let Ok(item) = item
            {
                content_map.insert(item.start, item);
            }
            else 
            {
                return Err(item.err().unwrap());
            }
        }
        let selector = Selector::parse("p:not(.I):not(.C):not(.T):not(.Z):not(.Y):not(.mark):not(.markx)").unwrap();
        let links_selector = Selector::parse("span[cmdprm]").unwrap();
        let par = html.select(&selector);
        let mut current_lvl = 0;
        let mut ci = Vec::new();
        for p in par
        {
            if let Some(str_id) = p.attr("id") && let Some(id ) = str_id.strip_prefix("p").and_then(|p| p.parse().ok())
            {
                let links: Vec<String> = p.select(&links_selector).into_iter().filter_map(|l|
                {
                    //<span class="cmd-hide" cmdprm="gohash=b113c2e08341853ef53a8dad4585b513d96f85e0f3d0d246a25ecf52e40608db goparaid=0 goback=0">Налогового кодекса Российской Федерации</span>
                    l.attr("cmdprm").and_then(|cmd| 
                        {
                            cmd.split_once(" ")
                                .and_then(|(left, _)| Some(left))
                                .and_then(|c|  Some(c.replace("gohash=", "")))
                        })
                }).collect();
                let links = if links.is_empty() {None} else { info!("Обрнаружены ссылки: {:?}", &links); Some(links) };
                
                let id: usize = id;
                if let Some(content_item) = content_map.get(&id)
                {
                    current_lvl = content_item.lvl;
                    let content = converter.convert(p.html());
                    let node = DocumentNode::new(&content_item.name, p.html(), content, links, content_item.start, content_item.end, content_item.lvl, &content_item.caption);
                    document_nodes.insert(node);
                    ci.push(content_item);
                }
                else 
                {
                    //надо проверять что он находиться в каком-то из диапазонов и только тогда добавлять а иначе вообще не добавлять
                    let content = converter.convert(p.html());
                    let node = DocumentNode::new("параграф", p.html(), content, links, id, id, current_lvl + 1, "параграф");
                    document_nodes.insert(node);
                }
            }
        }
        //tokio::fs::write("contents.json", &serde_json::to_string_pretty(&ci).unwrap()).await;
        Ok(document_nodes)

    }
}
#[cfg(test)]
mod tests
{
    use tracing::info;
    use utilites::Date;
    use crate::{converter, logger};

    struct NotConvert;
    impl converter::Converter<String> for NotConvert
    {
        fn convert(&self, html: String) -> String
        {
            html
        }
    }

    #[tokio::test]
    async fn test_data()
    {
        logger::init();
        let converter = NotConvert;
        let doc = super::SystemaClient::get_document(Date::new_date(31, 07, 2025), "287-ФЗ", converter).await.unwrap();
        let stats = doc.stats();
        info!("\nСтатистика дерева:");
        info!("Всего узлов: {}", stats.total_nodes);
        info!("Связей родитель-ребенок: {}", stats.total_children);
        let validation = doc.validate();
        println!("Результат валидации:");
        validation.print();
        for n in doc
        {
            if let Some(links) =  n.links_hashes()
            {
                info!("Обнаружены хеши ссылок: {:?}", links);
            }
        }
        //let d = serde_json::to_string_pretty(&doc).unwrap();
        //tokio::fs::write("test_doc.json", d).await;
        //info!("{:#?}", doc);
    }
}
