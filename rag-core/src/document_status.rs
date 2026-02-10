use serde::{Deserialize, Serialize};

/// Document status in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DocumentStatus
{
    /// Документ не загружен в базу данных
    NotLoaded,
    /// Документ загружен, но не обработан (не разбит на чанки и не добавлен в базу данных векторного поиска)
    Loaded,
    Embedded,
    Unknown
}
impl From<DocumentStatus> for u8
{
    fn from(value: DocumentStatus) -> Self 
    {
        match value
        {
            DocumentStatus::NotLoaded => 0,
            DocumentStatus::Loaded => 1,
            DocumentStatus::Embedded => 2,
            DocumentStatus::Unknown => u8::MAX
        }
    }
}

impl From<u8> for DocumentStatus
{
    fn from(value: u8) -> Self 
    {
        match value
        {
            0 => DocumentStatus::NotLoaded,
            1 => DocumentStatus::Loaded,
            2 => DocumentStatus::Embedded,
            _ => DocumentStatus::Unknown,
        }
    }
}
impl AsRef<str> for DocumentStatus
{
    fn as_ref(&self) -> &str 
    {
        match self
        {
            DocumentStatus::Loaded => "Loaded",
            DocumentStatus::Embedded => "Embedded",
            DocumentStatus::NotLoaded => "NotLoaded",
            DocumentStatus::Unknown => "Unknown"
        }
    }
}
