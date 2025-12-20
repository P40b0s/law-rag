use serde::{Deserialize, Serialize};
use utilites::{Date, http::Uri};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Document
{
    title: Vec<String>,
    uri: String,
    sign_date: String,
    number: String,
    sections: Vec<Section>
}
impl Document
{
    pub fn new(uri: &str, sign_date: String, number: &str) -> Self
    {
        Self 
        { 
            title: Vec::new(),
            uri: uri.to_owned(),
            sections: Vec::new(),
            sign_date,
            number: number.to_owned() 
        }
    }
    pub fn add_title(&mut self, title: String)
    {
        self.title.push(title);
    }
    pub fn add_section(&mut self, section: Section)
    {
        self.sections.push(section);
    }
     pub fn title(&self) -> &Vec<String> {
        &self.title
    }
    
    pub fn uri(&self) -> &str {
        &self.uri
    }
    
    pub fn sections(&self) -> &Vec<Section> {
        &self.sections
    }
    pub fn number(&self) -> &str {
        &self.number
    }
    pub fn date(&self) -> &str {
        &self.sign_date
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Section
{
    pub article: Option<String>,
    pub content: String
}
