mod ibpi_client;
mod error;
mod actual_redactions_client;
mod encoding;
mod models;
mod search_attributes;
mod document;
mod html_to_markdown;
use ::encoding::{DecoderTrap, Encoding, all::WINDOWS_1251};
pub use error::Error;
mod logger;
mod parser;
use crate::{actual_redactions_client::{ActualRedactionsClient, RedactionTtl}, document::{DocumentNode, DocumentNodes}, error::Result, html_to_markdown::HtmlToMarkdown, models::ContentItem};
use std::{cell::LazyCell, collections::BTreeMap, fmt::Display, sync::LazyLock};
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use utilites::{Date, Url, http::{ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, Bytes, HOST, HeaderName, HyperClient, REFERER, StatusCode, UPGRADE_INSECURE_REQUESTS, USER_AGENT, Uri}};


pub struct SystemaClient
{

}
impl SystemaClient
{
    ///Date::new_date(29, 05, 2024), "102-ФЗ"
    pub async fn get_document(sign_date: Date, number: &str) -> Result<DocumentNodes>
    {
        let (document, contents) = ActualRedactionsClient::get_document(sign_date, number).await?;
        let mut content_map = BTreeMap::new();
        let mut document_nodes = DocumentNodes::new();
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
        let par = document.select(&selector);
        let mut current_lvl = 0;
        let mut ci = Vec::new();
        for p in par
        {
            if let Some(str_id) = p.attr("id") && let Some(id ) = str_id.strip_prefix("p").and_then(|p| p.parse().ok())
            {
                let id: usize = id;
                if let Some(content_item) = content_map.get(&id)
                {
                    current_lvl = content_item.lvl;
                    let node = DocumentNode::new(&content_item.name, p.html(), content_item.start, content_item.end, content_item.lvl, &content_item.caption);
                    document_nodes.insert(node);
                    HtmlToMarkdown::parse(p.ancestors());
                    ci.push(content_item);
                }
                else 
                {
                    //надо проверять что он находиться в каком-то из диапазонов и только тогда добавлять а иначе вообще не добавлять
                    let node = DocumentNode::new("параграф", p.html(), id, id, current_lvl + 1, "параграф");
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

    use crate::logger;

    #[tokio::test]
    async fn test_data()
    {
        logger::init();
        let doc = super::SystemaClient::get_document(Date::new_date(31, 07, 2025), "287-ФЗ").await.unwrap();
        let stats = doc.stats();
        info!("\nСтатистика дерева:");
        info!("Всего узлов: {}", stats.total_nodes);
        info!("Связей родитель-ребенок: {}", stats.total_children);
        let validation = doc.validate();
        println!("Результат валидации:");
        validation.print();
        //let d = serde_json::to_string_pretty(&doc).unwrap();
        //tokio::fs::write("test_doc.json", d).await;
        //info!("{:#?}", doc);
    }
}
