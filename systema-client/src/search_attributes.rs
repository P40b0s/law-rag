use std::fmt::Display;
use serde::{ser::{SerializeMap, SerializeSeq}, Serialize, Serializer};
use serde_json::json;
use utilites::{http::Uri, Date};

use crate::{encoding::encode};
///Пришлось целый класс написать чтобы системовские поисковые атрибуты создать....
#[derive(Serialize)]
pub struct SearchAttributes
{
    #[serde(rename="AttrId")]
    attr_id: u32,
    #[serde(rename="AttrMode")]
    attr_mode: u32,
    #[serde(rename="IDParams")]
    #[serde(skip_serializing_if = "Option::is_none")]
    id_params: Option<Vec<IdParams>>,
    #[serde(rename="DateFrom")]
    #[serde(skip_serializing_if = "Option::is_none")]
    date_from: Option<String>,
    #[serde(rename="DateTo")]
    #[serde(skip_serializing_if = "Option::is_none")]
    date_to: Option<String>,
    #[serde(rename="Words")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "words_serializer")]
    ///второй варинт сделать Value и добавлять значения через макрос json!()
    words: Option<Vec<String>>,
}
///по умолчанию будем искать только ФЗ и ФКЗ
/// получиться такой json q
/// "IDParams":[{"Id":108,"Param":0},{"Id":107,"Param":0}]
#[derive(Serialize)]
struct IdParams
{
    #[serde(rename="Id")]
    id: u32,
    #[serde(rename="Param")]
    param: u32,
}

pub enum DocumentKind
{
    Fz,
    Fkz
}
impl DocumentKind
{
    pub fn as_id_param(&self) -> IdParams
    {
        match self 
        {
            DocumentKind::Fz => IdParams { id: 108, param: 0 },
            DocumentKind::Fkz => IdParams { id: 107, param: 0 },
        }
    }
}

impl SearchAttributes
{
    ///на вход принимается вектор из двух атрибутов  
    /// ```json
    /// [
    ///     {
    ///         "AttrId":5,
    ///         "AttrMode":0,
    ///         "DateFrom":"20240101",
    ///         "DateTo":"20240618"
    ///     },
    ///     {
    ///         "AttrId":999,
    ///         "AttrMode":1,
    ///         "Words":[50,"-date","20220701",0,1]
    ///     }
    /// ]
    /// ```
    /// 
    pub fn get_search_attributes_vec(date_from: Option<Date>, date_to: Date, kinds: &[DocumentKind], pages: u32, number: Option<&str>)-> Vec<Self>
    {

        let mut s = Vec::with_capacity(4);
        s.push
        (Self
            { 
                attr_id: 5,
                attr_mode: 0,
                date_from: date_from.and_then(|d| Some(d.format(utilites::DateFormat::JoinDate))),
                date_to: Some(date_to.format(utilites::DateFormat::JoinDate)),
                id_params: None,
                words: None
            });
        s.push
        (Self
            { 
                attr_id: 4,
                attr_mode: 1,
                date_from: None,
                date_to: None,
                id_params: Some(kinds.into_iter().map(|m| m.as_id_param()).collect()),
                words: None
            });
        if let Some(n) = number
        {
            s.push
            (Self
                { 
                    attr_id: 6,
                    attr_mode: 8,
                    date_from: None,
                    date_to: None,
                    id_params: None,
                    words: Some(vec![n.to_owned()])
                });
        }
        s.push
        (Self
            { 
                attr_id: 999,
                attr_mode: 1,
                date_from: None,
                date_to: None,
                id_params: None,
                words: Some(vec![pages.to_string(), "-date".to_owned(), "20220701".to_owned(), "0".to_owned(), "1".to_owned()])
            });
        s
    }
    /// http://actual.pravo.gov.ru:8000/api/ebpi/attrsearch/?q=[{"AttrId":5,"AttrMode":0,"DateFrom":"20240101","DateTo":"20240620"},{"AttrId":999,"AttrMode":1,"Words":[50,"-date","20220701",0,1]}]
    /// только кавычки в эскейпе -> %22
    pub fn get_search_uri(date_from: Option<Date>, date_to: Date, kinds: &[DocumentKind], pages: u32, number: Option<&str>)-> String
    {
        let v = Self::get_search_attributes_vec(date_from, date_to,  kinds, pages, number);
        let attrs = serde_json::to_string(&v).unwrap();
        let url = ["/attrsearch/?q=", &encode(&attrs)].concat().parse().unwrap();
        url
    }
    
}

fn words_serializer<S>(words: &Option<Vec<String>>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if words.is_none()
    {
        let s = s.serialize_map(None)?;
        s.end()
    }
    else 
    {
        let words = words.as_ref().unwrap();
        let mut ser = s.serialize_seq(Some(words.len()))?;
        if words.len() == 1
        {
            
            let number = words[0].as_str();
            let _ = ser.serialize_element(number);
        }
        else if words.len() == 5
        {
            let pages:u32 = words[0].as_str().parse().unwrap();
            let sorting_order = words[1].as_str();
            //какая то дата хз
            let some_date = words[2].as_str();
            let xz1:u32 = words[3].as_str().parse().unwrap();
            let xz2:u32 = words[4].as_str().parse().unwrap();
            let _ = ser.serialize_element(&pages);
            let _ = ser.serialize_element(&sorting_order);
            let _ = ser.serialize_element(&some_date);
            let _ = ser.serialize_element(&xz1);
            let _ = ser.serialize_element(&xz2);
           
        }
        ser.end()
    }
   
}