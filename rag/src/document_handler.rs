use scraper::{ElementRef, Selector, element_ref::Text};
use systema_client::{DocumentKindSearchParams, SystemaIpsApi};
use tracing::info;
use utilites::Date;
use crate::{document::{Document, Section}};
use crate::error::{Error, Result};

pub struct DocumentHandler
{

}
impl DocumentHandler
{
    ///"273-фз"
    ///Date::new_date(29, 12, 2012)
    pub async fn parse_document(number: &str, sign_date: Date) -> Result<Document>
    {
        let sign_date_str = sign_date.format(utilites::DateFormat::SerializeDate);
        let document = SystemaIpsApi::search(
            &[DocumentKindSearchParams::Fz, DocumentKindSearchParams::Fkz],
            number,
            sign_date).await?;
        let html = document.get_document_html().await?;
        let current_uri = document.current_uri();
        let body_selector = Selector::parse("body").unwrap();
        let mut current_article: Option<String> = None;
        if let Some(body) = html.select(&body_selector).next()
        {
            let mut document = Document::new(document.current_uri(), sign_date_str, number);
            for node in body.children() 
            {
                let mut current_section: Option<Section> = None;
                if let Some(element) = node.value().as_element() 
                {
                    if let Some(element_ref) = ElementRef::wrap(node) 
                    {
                        let tag_name = element.name();
                        let text = element_ref.text().collect::<String>().trim().to_string();
                        info!("Обработка тега: {}, текст: {}", tag_name, text);
                        if text.is_empty() || text == " " || text == "&nbsp;" 
                        {
                            continue;
                        }
                        else if element.classes().any(|a| a == "T")
                        {
                            document.add_title(element_ref.text().collect());
                        } 
                        else if element.classes().any(|a| a == "H") && element_ref.text().collect::<String>().starts_with("Статья")
                        {
                            let article_text: String = element_ref.text().collect();
                            current_article = Some(article_text);
                        } 
                        else if element.classes().count() == 0 && tag_name == "p"
                        {
                            current_section = Some( Section { article: current_article.clone(), content: Self::content_handler(element_ref.text())});
                        }
                        if let Some(cs) = current_section
                        {
                            document.add_section(cs);
                        }
                    }
                }
            }
           
            Ok(document)
        }
        else 
        {
            Err(Error::ParserError { context: "tag `body` not found in document".to_owned() , uri: current_uri.to_owned() })
        }
    }

    fn content_handler<'a>(text: Text<'a>) -> String
    {
        let content: String = text.collect();
        content.replace("\u{a0}", " ")
    }
}

#[cfg(test)]
mod tests
{
    use tracing::info;
    use utilites::Date;

    use crate::logger;

    #[tokio::test]
    async fn test_handler()
    {
        logger::init();
        let date = Date::new_date(29, 12, 2012);
        let number = "273-фз";
        let handler = super::DocumentHandler::parse_document(number, date).await.unwrap();
        info!("{:#?}", handler);

    }
}