use std::error::Error as StdError;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error
{
    #[error(transparent)]
    QdrantError(#[from] qdrant_client::QdrantError),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    #[error("Vector size mismatch: expected {0}, got {1}")]
    VectorSizeError(usize, usize),
    #[error(transparent)]
    EmbeddingsError
    {
        #[from]
        source: embedding::Error
    },
}
