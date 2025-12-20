use std::error::Error as StdError;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error
{
    // #[error(transparent)]
    // ApiError
    // {
    //     #[from]
    //     source: Box<dyn std::error::Error + Send + Sync>,
    // }
    #[error("Systema api error: `{0}`")]
    ApiError(String),
    #[error(transparent)]
    UtilitesError(#[from] utilites::error::Error),
    #[error("parse htmp error: `{0}`")]
    ScraperError(String)
}

impl Error
{
    pub fn api_error(err: &str) -> Self
    {
        Error::ApiError(err.to_owned())
    }
}