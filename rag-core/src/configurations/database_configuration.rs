use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QdrantConfiguration
{
    #[serde(default = "min_search_score")]
    pub min_search_score: f32,
    #[serde(default = "qdrant_url")]
    pub qdrant_url: String,
    #[serde(default = "distance")]
    pub distance: String
}

fn min_search_score() -> f32
{
    0.5
}

fn qdrant_url() -> String
{
    "http://127.0.0.1:6334".to_owned()
}

fn distance() -> String
{
    "cosine".to_owned()
}

impl Default for QdrantConfiguration
{
    fn default() -> Self 
    {
        Self
        {
            min_search_score: min_search_score(),
            qdrant_url: qdrant_url(),
            distance: distance()
        }
    }
}
