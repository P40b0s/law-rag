use std::error::Error as StdError;

use tokio::task::JoinError;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error
{
    #[error(transparent)]
    ApiError(#[from] systema_client::Error),
    #[error("Content structure error: `{0}`")]
    ContentError(String),
    #[error(transparent)]
    UtilitesError(#[from] utilites::error::Error),
    #[error("parse html error: `{0}`")]
    ScraperError(String),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::error::Error),
    #[error(transparent)]
    Tokenizer
    {
        #[from]
        source: tokenizers::Error
    },
    #[error(transparent)]
    IoError
    {
        #[from]
        source: std::io::Error
    },
    #[error(transparent)]
    CandleError
    {
        #[from]
        source:  candle_core::Error
    },
    #[error(transparent)]
    TokioTaskError
    {
        #[from]
        source:  JoinError
    }
   
}

