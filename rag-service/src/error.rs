use std::error::Error as StdError;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error
{
    #[error(transparent)]
    DatabaseError(#[from] database::Error),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    ApiError(#[from] systema_client::Error),
    #[error(transparent)]
    EmbeddingsError
    {
        #[from]
        source: embedding::Error
    },
    #[error("Модель retriver не была загружена")]
    RetriverModelLoadError,
    #[error("Модель reranker не была загружена")]
    RerankerModelLoadError,
    #[error("Ошибка загрузки модели генератора `{source}`")]
    GeneratorModelLoadError
    {
        source: anyhow::Error
    },
    #[error("Модель генератора не была загружена")]
    GeneratorModelNotLoadError,
    #[error("Ошибка загрузки модели `{source}`")]
    EmbeddingModelLoadError
    {
        source: anyhow::Error
    }

}
