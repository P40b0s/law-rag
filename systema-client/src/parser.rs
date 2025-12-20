use scraper::{ElementRef, Html, Selector};
use tracing::info;
use crate::error::{Error, Result};

pub fn get_document_body<'a>(body: String) -> Result<Html, Error>
{
    let document = Html::parse_document(&body);
    let body_selector = Selector::parse("#text_content").map_err(|e| Error::ScraperError(e.to_string()))?;
    let body = document.select(&body_selector).next().ok_or(Error::ScraperError("Selector `#text_content` not found".to_owned()))?;
    let document = Html::parse_document(&body.inner_html());
    // let body_selector = Selector::parse("body").unwrap();
    // let body = document.select(&body_selector).next().unwrap();
    Ok(document)
}