mod layers;
mod router;
mod static_files_router;
mod documents;
mod types;
pub use static_files_router::static_router;
pub use router::router;
use serde::{Deserialize, Serialize};
use utilites::Date;
use uuid::Uuid;


#[derive(Debug, Deserialize)]
pub struct DocumentRequest
{
    sign_date: Date,
    number: String,
    collection_id: Uuid
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingRequest
{
    hash: String,
    collection_id: Uuid
}
#[derive(Debug, Deserialize)]
pub struct GenerationRequest
{
    query: String,
    final_limit: usize,
    per_collection_limit: usize
}
#[derive(Debug, Deserialize)]
pub struct CollectionAddRequest
{
    name: String,
    description: String,
}
#[derive(Debug, Deserialize)]
pub struct CollectionUpdateRequest
{
    id: Uuid,
    description: String,
}

pub fn combine_path(path: &'static str) -> String
{
    ["api_v1/", path].concat()
}
pub fn with_api_version(version: ApiVersion, path: &'static str) -> String
{
    [version.as_ref(), path].concat()
}
/// /api_v{version number}
pub enum ApiVersion 
{
    V1,
    V2
}
impl AsRef<str> for ApiVersion
{
    fn as_ref(&self) -> &str 
    {
        match self 
        {
            ApiVersion::V1 => "/api_v1",
            ApiVersion::V2 => "/api_v2"    
        }
    }
}
