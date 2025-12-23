use std::sync::LazyLock;

use rangemap::RangeInclusiveMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use utilites::{empty_string_as_none, null_string_as_none, Date, deserialize_date};

use crate::Error;




#[derive(Debug, Deserialize)]
pub struct RedactionsResponse
{
    #[serde(rename="docid")]
    pub id: u32,
    #[serde(rename="dochash")]
    pub hash: String,
    #[serde(rename="serverdate")]
    #[serde(deserialize_with = "deserialize_date")]
    pub server_date: utilites::Date,
    pub redactions: Vec<Redaction>,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub error: Option<String>,
}

///редакция
#[derive(Debug, Deserialize)]
pub struct Redaction
{
    /// redid	444772 по этому id редакции можно запросить текст документа
    #[serde(rename="redid")]
    pub id: u32,
    #[serde(rename="reddocrefid")]
    pub ref_id: u32,
    #[serde(rename="reddate")]
    #[serde(deserialize_with="deserialize_date")]
    pub date: Date,
    #[serde(rename="reddatetimed")]
    #[serde(deserialize_with="deserialize_date")]
    pub date_time: Date,
    #[serde(rename="redstateid")]
    pub state_id: u32,
    #[serde(rename="stateclass")]
    pub state_class: u32,
    ///"Действует с изменениями"
    #[serde(rename="statename")]
    pub state: String,
    #[serde(rename="redelements")]
    pub elements:	i16,
    #[serde(rename="redtype")]
    ///-1 - редакции которые вступают в силу ранее даты подписания которая храниться в поле date_time  
    /// 0 - редакции которые вступают в силу при насуплении указанной даты (date_time)  
    /// 1 - редакции которые вступают в силу при наступлении неизвестной даты для них установлено значение date_time = 20990101  
    /// 2 - зомби-редакции, которые не вступают в силу при наступлении указанной даты, они не считаются последующими или будущими редакциями документа, их как бы нет. 
    pub redaction_type: u32,
    ///	" актуальная"
    #[serde(rename="redcaption")]
    pub caption: String,
    #[serde(rename="redstatus")]
    ///	"актуальная" не действующая действующая не всупившая
    pub status: String,
    #[serde(deserialize_with = "empty_string_as_none")]
    #[serde(rename="redreason")]
    pub reason: Option<String>,
    ///57
    #[serde(rename="redflags")]
    pub flag:	u32,
    #[serde(rename="redcompleted")]
    ///редакция завершена
    pub is_completed: bool,//	true
    #[serde(rename="redchecked")]
    ///редация проверена (Кем?)
    pub is_checked: bool,//	true
    #[serde(rename="redofficial")]
    pub is_official: bool,//	true
    #[serde(rename="actual")]
    pub is_actual: bool,//	true
    #[serde(rename="redinitial")]
    pub is_initial: bool,//	false
    #[serde(rename="hascontent")]
    ///Наличие оглавления mna2022
    pub is_has_content: bool,//	true
    #[serde(rename="contentcomplete")]
    ///Наличие оглавления mna2022
    pub is_content_complete: bool,//	true
}



#[derive(Debug, Deserialize)]
pub struct DocumentsSearchResponse
{
    pub docs: Vec<SystemaDocumentCard>,
    pub docscount: u32,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub error: Option<String>
}

#[derive(Debug, Deserialize, Clone)]
pub struct SystemaDocumentCard
{
    /// 305255
    #[serde(rename="docid")]
    pub doc_id: u32,
    /// поле пустое, непонятно что тут должно быть
    #[serde(rename="docstampname")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub stamp_name: Option<String>,
    /// "О структуре федеральных органов исполнительной власти"
    #[serde(rename="docnames")]
    pub name: String,
    /// поле пустое, непонятно что тут должно быть
    #[serde(rename="docdescription")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub description: Option<String>,
    /// "Указ Президента Российской Федерации от 11.05.2024 № 326"
    #[serde(rename="docpassing")]
    pub complex_name: String,
    /// "Действует с изменениями"
    #[serde(rename="docstate")]
    pub doc_state: String,
    /// "http://publication.pravo.gov.ru/Document/View/0001202405110002"
    #[serde(rename="docimagepath")]
    pub publication_url: String,
    /// '"Российская газета" от 13.05.2024' итд...
    pub publications: Vec<String>,
    /// null
    #[serde(rename="dockind")]
    #[serde(deserialize_with = "null_string_as_none")]
    pub kind: Option<String>,
    /// null
    #[serde(rename="kindrank")]
    #[serde(deserialize_with = "null_string_as_none")]
    pub kind_rank: Option<String>,
    /// "20240511"
    #[serde(rename="docpass0date")]
    #[serde(deserialize_with="deserialize_date")]
    pub sign_date: Date,
    /// 326
    #[serde(rename="docpass0numberint")]
    pub number_int: u32,
    /// был бы фз был бы "326-ФЗ"
    #[serde(rename="docpass0number")]
    pub number: String,
    /// "cc4a7f7b594d929a90706d741ac9cf532ace317526834f447188dba16999c1aa"
    #[serde(rename="dochash")]
    pub hash: String,
}




#[derive(Debug, Deserialize)]
pub struct SystemaTextResponse
{
    /// html документа
    #[serde(rename="redtext")]
    pub text_html: String,
    ///ошибка, например если документ не найден будет "Докцумент не найден"
    #[serde(deserialize_with = "null_string_as_none")]
    pub error: Option<String>,
}




// id: 455520,
// ref_id: 307408,
// date: Date(2025-03-01T00:00:00),
// date_time: Date(2025-03-01T05:37:30),
// state_id: 3,
// state_class: 3,
// state: "Действует с изменениями",
// elements: 925,
// redaction_type: 0,
// caption: "153. на 01.03.2025 (№ 171-ФЗ от 08.07.2024), с изменениями, не вступившими в силу",
// status: "не вступившая",
// reason: None,
// flag: 58,
// is_completed: true,
// is_checked: false,
// is_official: false,
// is_actual: false,
// is_initial: false,
// is_has_content: true,
// is_content_complete: true
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtendedRedaction
{
    /// redid	444772 по этому id редакции можно запросить текст документа
    pub id: u32,
    pub date: Date,
    /// с изменениями, не вступившими в силу - 3
    /// 
    pub state_id: u32,
    ///"Действует с изменениями"
    pub state: String,
    pub elements:	i16,
    ///-1 - редакции которые вступают в силу ранее даты подписания которая храниться в поле date_time  
    /// 0 - редакции которые вступают в силу при насуплении указанной даты (date_time)  
    /// 1 - редакции которые вступают в силу при наступлении неизвестной даты для них установлено значение date_time = 20990101  
    /// 2 - зомби-редакции, которые не вступают в силу при наступлении указанной даты, они не считаются последующими или будущими редакциями документа, их как бы нет. 
    pub redaction_type: u32,
    ///	" актуальная"
    pub caption: String,
    ///	"актуальная" недействующая, действующая не всупившая
    pub status: String,
    ///57
    pub flag:	u32,
    pub is_actual: bool,
    ///номер документа который вносит изменение
    pub source_number: Option<String>,
    ///дата документа который вносит изменение
    pub source_date: Option<Date>,
}
static REDACTIONS_RX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{1,}[.]\s+([*]\s+)?\w{2}\s+(?<red_date>\d{2}[.]\d{2}[.]\d{4})\s+[(](№\s+(?<source_number>\d{1,}-[ФЗК]+)(\s+\w{2}\s+(?<source_date>\d{2}[.]\d{2}[.]\d{4}))?)[)]").unwrap());
impl From<Redaction> for ExtendedRedaction
{
    fn from(value: Redaction) -> Self 
    {
        let captures =  if let Some(cpt) = REDACTIONS_RX.captures(&value.caption)
        {
            let source_date = cpt.name("source_date").and_then(|v| Date::parse(v.as_str()));
            let source_number = cpt.name("source_number").and_then(|v| Some(v.as_str().to_owned()));
            (source_date, source_number)
        }
        else
        {
            (None, None)
        };
        Self 
        { 
            id: value.id,
            date: value.date,
            state_id: value.state_id,
            state: value.state,
            elements: value.elements,
            redaction_type: value.redaction_type,
            caption: value.caption,
            status: value.status,
            flag: value.flag,
            is_actual: value.is_actual,
            source_date: captures.0,
            source_number: captures.1
        }
    }
}

///http://actual.pravo.gov.ru:8000/api/ebpi/getcontent/?bpa=ebpi&rdk=483442
/// содержание
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Contents
{
    #[serde(rename="data")]
    pub content: Vec<Content>,
    pub error: Option<String>,
    pub status: u8,
    ///Федеральный закон
    pub typeact: String,
    pub lockkey: usize
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content
{
    /// id a1, a1_j1 хз что это
    pub id: String,
    ///начало текущего блока `p8`
    #[serde(rename="np")]
    pub paragraph_start_number: String,
    //конец блока `p14`
    #[serde(rename="npe")]
    pub paragraph_end_number: String,
    ///`$пункт 1`
    pub caption: String,
    ///`пункт`
    pub unit: String,
    ///уровень `1`
    pub lvl: usize
}
impl TryFrom<Content> for ContentItem
{
    type Error = Error;
    fn try_from(value: Content) -> Result<Self, Self::Error> 
    {
        let start_index: Option<usize> = value.paragraph_start_number.strip_prefix("p").and_then(|p| p.parse().ok());
        let end_index: Option<usize> = value.paragraph_end_number.strip_prefix("p").and_then(|p| p.parse().ok());
        if let Some(start) = start_index && let Some(end) = end_index
        {
            Ok(Self
            {
                start,
                end,
                caption: value.caption,
                name: value.unit,
                lvl: value.lvl
            })
        }
        else 
        {
            Err(Error::ContentError(format!("Ошибка получения данных из содержания документа! np:{} npe:{}", value.paragraph_start_number, value.paragraph_end_number)))    
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContentItem
{
    pub start: usize,
    pub end: usize,
    ///`$пункт 1`
    pub caption: String,
    ///`пункт`
    pub name: String,
    ///уровень `1`
    pub lvl: usize
}


#[cfg(test)]
mod tests
{

   
}