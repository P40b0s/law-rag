
// use std::{fmt::Display};

// use hyper::Uri;
// use serde::{ser::{SerializeMap, SerializeSeq}, Serialize, Serializer};
// use serde_json::json;
// use utilites::Date;
// use crate::{encoding::encode, models::{Redaction, SystemaDocumentCard}, request::empty_get_request, search_attributes::SearchAttributes, SystemaApiError, ACTUAL_BACKEND_URL, ACTUAL_EBPI_PATH};

// ///TODO сделать парсер доков из IPS у меня уже есть похожий - https://github.com/P40b0s/format_constructor/blob/master/format_parser/src/from_html/source.rs
// ///http://pravo.gov.ru/proxy/ips/?searchres=&bpas=cd00000&a3=102000505%3B102000506&a3type=1&a3value=&a6=&a6type=1&a6value=&a15=&a15type=1&a15value=&a7type=1&a7from=&a7to=&a7date=29.12.2022&a8=573-%D4%C7&a8type=1&a1=&a0=&a16=&a16type=1&a16value=&a17=&a17type=1&a17value=&a4=&a4type=1&a4value=&a23=&a23type=1&a23value=&textpres=&sort=7&x=64&y=19
// ///  let base_page = "http://pravo.gov.ru/proxy/ips/";
// // let request = [
// //     base_page,
// //     "?list_itself=&bpas=cd00000&a3=102000505&a3type=1&a3value=&a6=&a6type=1&a6value=&a15=&a15type=1&a15value=&a7type=1&a7from=&a7to=&a7date=",
// //     date,
// //     "&a8=",
// //     number,
// //     "-%D4%C7&a8type=1&a1=&a0=&a16=&a16type=1&a16value=&a17=&a17type=1&a17value=&a4=&a4type=1&a4value=&a23=&a23type=1&a23value=&textpres=&sort=7&x=19&y=7"].concat();

mod error;
pub use error::Error;
mod logger;
mod parser;
use crate::error::{Result};
use std::{cell::LazyCell, fmt::Display, sync::LazyLock};
use encoding::{all::WINDOWS_1251, DecoderTrap, Encoding};
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tracing::info;
use utilites::{Date, Url, http::{ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, Bytes, HOST, HeaderName, HyperClient, REFERER, StatusCode, UPGRADE_INSECURE_REQUESTS, USER_AGENT, Uri}};

//use crate::{encoding::encode, SystemaApiError};

pub static REDACTIONS_RX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d{1,}\s+-\s+\w{2}\s+(?<date>\d{2}[.]\d{2}[.]\d{4})\s+(№\s+(?<number>\d{1,}-[ФЗК]+))?\s+([(](?<comment>[^)]+))?").unwrap());
const BASE: &str = "http://pravo.gov.ru/proxy/ips/";
pub enum DocumentKindSearchParams
{
    Fz,
    Fkz
}
impl Display for DocumentKindSearchParams
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            DocumentKindSearchParams::Fz => f.write_str("102000505"),
            DocumentKindSearchParams::Fkz => f.write_str("102000506"),
        }
    }
}

/// value: 37,102162745, text: 37 - от 03.07.2016 № 306-ФЗ (изм.)  
/// value: n, text: 38 - от 03.07.2016 № 313-ФЗ (изм.)(не готова)
#[derive(Debug,Serialize, Deserialize, Clone)]
pub struct Edition
{
    ///Порядковый номер редакции
    pub edition_id: Option<u32>,
    pub is_ready: bool,
    pub edition_date: Option<Date>,
    pub edition_changed_by_number: Option<String>,
    pub comment: Option<String>
}
#[derive(Debug,Serialize, Deserialize, Clone)]
pub struct Editions
{
    pub doc_id: String,
    pub editions: Vec<Edition>
}

impl Editions
{
    pub fn new(doc_id: &str, editions: &[(&str, String)]) -> Self
    {
        let mut new_editions: Vec<Edition> = Vec::with_capacity(editions.len());
        for e in editions
        {
            let id: Vec<&str> = e.0.split_terminator(",").collect();
            if !id.is_empty()
            {
                let id = id[0];
                let id = if id == "n"
                {
                    None
                }
                else 
                {
                    id.parse::<u32>().ok()
                };
                for cpt in REDACTIONS_RX.captures_iter(&e.1)
                {
                    let date = cpt.name("date").and_then(|v| Date::parse(v.as_str()));
                    let number = cpt.name("number").and_then(|v| Some(v.as_str()));
                    let comment = cpt.name("comment").and_then(|v| Some(v.as_str()));
                    let edition = Edition 
                    {
                        edition_id: id,
                        is_ready: id.is_some(),
                        edition_date: date,
                        edition_changed_by_number: number.and_then(|v| Some(v.to_owned())),
                        comment: comment.and_then(|v| Some(v.to_owned())),
                    };
                    new_editions.push(edition);
                }
            }
        }
        Self{doc_id: doc_id.to_owned(), editions: new_editions}
    }
    pub fn get_editions(&self) -> &[Edition]
    {
        &self.editions
    }
    pub fn get_doc_id(&self) -> &str
    {
        &self.doc_id
    }
    ///Дата на которую редакция должна быть готова  
    /// и номер документа по которому эта редакция создана
    pub fn get_edition(&self, edition_date: Date, number: &str) -> Option<Edition>
    {
        let f = self.editions.iter().find(|e|
        {
            e.edition_date.as_ref().is_some_and(|e| e.date_is_equalis(&edition_date))
            && e.edition_changed_by_number.as_ref().is_some_and(|n| n == &number)
        });
        f.cloned()
    }
}
pub struct SystemaIpsApi
{
    uri: String
}
/// FIXME ТУТ ВСЕ РАБОТАЕТ, НАДО ПОМЕНЯТЬ КЛИЕНТА! ПОКА В ЭТОМ АПИ НЕТ НЕОБХОДИМОСТИ!
//http://95.173.147.130/proxy/ips/?list_itself=&bpas=cd00000&a3=102000505;102000506&a3type=1&a3value=&a6=&a6type=1&a6value=&a15=&a15type=1&a15value=&a7type=1&a7from=&a7to=&a7date=29.12.2012&a8=273-%F4%E7&a8type=1&a1=&a0=&a16=&a16type=1&a16value=&a17=&a17type=1&a17value=&a4=&a4type=1&a4value=&a23=&a23type=1&a23value=&textpres=&sort=7&x=49&y=9&page=firstlast
impl SystemaIpsApi
{

    fn client() -> HyperClient
    {
        HyperClient::new(BASE.parse().unwrap()).with_headers(Self::headers())
    }
    fn headers() -> Vec<(HeaderName, String)>
    {
        let mut h= Vec::new();
        h.push((HOST, "pravo.gov.ru".to_owned()));
        h.push((USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64; rv:127.0) Gecko/20100101 Firefox/127.0".to_owned()));
        h.push((ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".to_owned()));
        h.push((ACCEPT_ENCODING, "gzip, deflate".to_owned()));
        h.push((ACCEPT_LANGUAGE, "ru-RU,ru;q=0.8,en-US;q=0.5,en;q=0.3".to_owned()));
        h.push((REFERER, "http:://pravo.gov.ru".to_owned()));
        h.push((UPGRADE_INSECURE_REQUESTS, "1".to_owned()));
        h
    }

    ///Проверка что пришел код 200 на запрос
    fn code_error_check(response: (StatusCode, Bytes)) -> Result<Bytes>
    {
        if response.0 != utilites::http::StatusCode::OK
        {
            let e = ["Сервер ответил кодом ", response.0.as_str(), " ожидался код 200"].concat();
            tracing::warn!("{}", &e);
            return Err(Error::ApiError(e));
        }
        else 
        {
            Ok(response.1)
        }
    }

    fn search_uri(doc_types: &[DocumentKindSearchParams], doc_number: &str, sign_date: Date) -> String
    {
        let doc_types = doc_types.iter().map(|dt| dt.to_string()).collect::<Vec<String>>().join(";");
        //некрасиво, но непонятно как они кодируют url
        let doc_number = doc_number.to_lowercase().replace("фз", "%F4%E7").replace("фкз", "%F4%EA%E7");
        let search_request = [ 
        "?list_itself=", 
        "&bpas=cd00000",
        "&a3=", &doc_types,
        "&a3type=1",
        "&a3value=",
        "&a6=",
        "&a6type=1",
        "&a6value=",
        "&a15=",
        "&a15type=1",
        "&a15value=",
        "&a7type=1",
        "&a7from=",
        "&a7to=",
        "&a7date=", &sign_date.format(utilites::DateFormat::DotDate),
        "&a8=", doc_number.as_ref(),
        "&a8type=1",
        "&a1=",
        "&a0=",
        "&a16=",
        "&a16type=1",
        "&a16value=",
        "&a17=",
        "&a17type=1",
        "&a17value=",
        "&a4=",
        "&a4type=1",
        "&a4value=",
        "&a23=",
        "&a23type=1",
        "&a23value=",
        "&textpres=",
        "&sort=7",
        "&x=49",
        "&y=9"].concat();
        search_request
    }

    async fn get_document_id(doc_types: &[DocumentKindSearchParams], doc_number: &str, sign_date: Date) -> Result<String>
    {
        let mut client = Self::client();
        client = client.add_path(&Self::search_uri(doc_types, doc_number, sign_date.clone()));
        info!("request uri: {:?}", client.get_uri());
        let (code, data) = client.get().await?;
        match code
        {
            StatusCode::OK => (),
            StatusCode::NO_CONTENT => return Err(Error::ApiError(["Документ ", doc_number, " ", &sign_date.to_string(), " не найден!"].concat())),
            _ => return Err(Error::ApiError(format!("статус запроса {}", code)))

        }
        let body = Self::enc_win1251(&data)?;
        let page = Html::parse_document(&body);
        let selector = Selector::parse(r#"a[id="link_0"]"#).unwrap();
        if let Some(element) = page.select(&selector).next()
        {
            let href = element.value().attr("href");
            if href.is_none()
            {
                let err = ["Ссылка на тело документа № ", doc_number, " от ", &sign_date.to_string(), " не найдена"].concat();
                tracing::error!("{}", &err);
                return Err(Error::ApiError(err));
            }
            let href = href.unwrap();
            //получаем номер дока из этого:
            //?docbody=&link_id=0&nd=102162745&intelsearch=
            if let Some(nd) = href.split_once("nd")
            {
                let nd = nd.1.split_once("&").unwrap();
                let id = nd.0.split_once("=").unwrap().1;
                tracing::debug!("номер документа -> {}", id);
                Ok(id.to_owned())
            }
            else
            {
                return Err(Error::api_error("В списке документов не найден документ с идентификатором `nd`"));
            }
        }
        else 
        {
            return Err(Error::api_error("По текущим параметрам не найдено ни одного документа"));
        }
    }

    async fn get_editions(doc_types: &[DocumentKindSearchParams], doc_number: &str, sign_date: Date) -> Result<Editions>
    {
        let id = Self::get_document_id(doc_types, doc_number, sign_date).await?;
        Self::get_editions_by_doc_id(&id).await
    }
    async fn get_editions_by_doc_id(doc_id: &str) -> Result<Editions>
    {
        let doc_uri = ["?docbody=&link_id=0&nd=", &doc_id, "&intelsearch=&firstDoc=1"].concat();
        let mut client = Self::client();
        client = client.add_path(&doc_uri);
        let response = client.get().await?;
        let document = Self::code_error_check(response)?;
        let redactions_html = Self::enc_win1251(&document)?;
        let red_page = Html::parse_document(&redactions_html);
        let selector = Selector::parse(r#"select[name="doc_editions"]"#).unwrap();
        if let Some(element) = red_page.select(&selector).next()
        {
            let selector = Selector::parse(r#"option"#).unwrap();
            let mut options: Vec<(&str, String)> = vec![];
            for option in element.select(&selector)
            {
                if let Some(val) = option.attr("value")
                {
                    options.push((val, option.inner_html()));
                    tracing::debug!("value:{}->text:{}", val,  option.inner_html());
                }
            }
            return Ok(Editions::new(doc_id, &options));
        }
        Err(Error::ApiError("Не найден тэг select[name=\"doc_editions\"], невозможно найти редакции для документа".to_owned()))
        //http://95.173.147.130/proxy/ips/?docbody=&link_id=0&nd=102162745&intelsearch=&firstDoc=1
    }
    pub async fn get_document(&self) -> Result<String>
    {
        let mut client = Self::client();
        client = client.add_path(&self.uri);
        let response = client.get().await?;
        let document = Self::code_error_check(response)?;
        let doc_html = Self::enc_win1251(&document)?;
        Ok(doc_html)
    }
    pub async fn get_document_html(&self) -> Result<Html>
    {
        let doc = self.get_document().await?;
        let doc = parser::get_document_body(doc)?;
        Ok(doc)
    }
    pub fn current_uri(&self) -> &str
    {
        &self.uri
    }

    pub async fn search(doc_types: &[DocumentKindSearchParams], doc_number: &str, sign_date: Date) -> Result<Self>
    {
        let id = Self::get_document_id(doc_types, doc_number, sign_date).await?;
        Ok(Self { uri: ["?doc_itself=&nd=", &id, "&page=1", "&fulltext=1"].concat()})
    }

    fn enc_win1251(bytes: &[u8]) -> Result<String>
    {
        let result = WINDOWS_1251.decode(&bytes, DecoderTrap::Strict);
        if result.is_err()
        {
            return Err(Error::ApiError(format!("Ошибка открытия html ответа в кодировке windows-1251 {}", result.as_ref().err().unwrap())));
        }
        let result = result.unwrap();
        Ok(result)
    }
}
#[cfg(test)]
mod tests
{
    use utilites::Date;

    use crate::{SystemaIpsApi, logger, parser};

    use super::DocumentKindSearchParams;
    #[test]
    fn test_uri()
    {
        logger::init();
        let s = super::SystemaIpsApi::search_uri(
            &[DocumentKindSearchParams::Fz, DocumentKindSearchParams::Fkz],
            "273-фз",
            Date::new_date(29, 12, 2012));
        assert_eq!("?searchres=&bpas=cd00000&a3=102000505;102000506&a3type=1&a3value=&a6=&a6type=1&a6value=&a15=&a15type=1&a15value=&a7type=1&a7from=&a7to=&a7date=29.12.2012&a8=273-%F4%E7&a8type=1&a1=&a0=&a16=&a16type=1&a16value=&a17=&a17type=1&a17value=&a4=&a4type=1&a4value=&a23=&a23type=1&a23value=&textpres=&sort=7&x=49&y=9",
        &s);
        tracing::info!("{s}");
    }

    #[tokio::test]
    async fn test_search_doc()
    {
        logger::init();
        let doc = SystemaIpsApi::search(&[DocumentKindSearchParams::Fz, DocumentKindSearchParams::Fkz],
            "273-фз",
            Date::new_date(29, 12, 2012)).await.unwrap().get_document().await;
       
        //parser::get_document_body(s.unwrap());
        //tracing::info!("Найден документ: {:?}", s);
    }

    #[tokio::test]
    async fn test_get_editions()
    {
        logger::init();
        let s = super::SystemaIpsApi::get_editions(
            &[DocumentKindSearchParams::Fz, DocumentKindSearchParams::Fkz],
            "273-фз",
            Date::new_date(29, 12, 2012)).await.unwrap();
        let editions = s.get_editions();
        for e in editions
        {
            tracing::info!("{:?}", e);
        }
        
    }
}