use std::error::Error as StdError;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error
{
    #[error(transparent)]
    DatabaseError(#[from] database::Error),
    #[error(transparent)]
    ApiError(#[from] systema_client::Error),
    #[error(transparent)]
    EmbeddingsError
    {
        #[from]
        source: embedding::Error
    },
}
