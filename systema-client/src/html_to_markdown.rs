use scraper::{ElementRef, Html};
use tracing::info;

pub struct HtmlToMarkdown
{

}
impl HtmlToMarkdown
{
    pub fn parse<'a, I: Iterator<Item = ElementRef<'a>>>(element: I)
    {
        for el in element
        {
            info!("ELEMENT: {:?}", el);
        }
    }
}