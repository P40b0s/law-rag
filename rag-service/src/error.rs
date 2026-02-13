use std::error::Error as StdError;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error
{
    #[error(transparent)]
    DatabaseError(#[from] database::Error),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Ошибка выполенния потока `{0}`")]
    ThreadError(anyhow::Error),
    #[error(transparent)]
    EmbeddingsError
    {
        #[from]
        source: embedding::Error
    },
    #[error("Модель retriver не была загружена")]
    RetriverModelNotLoad,
    #[error("Модель reranker не была загружена")]
    RerankerModelNotLoad,
    #[error("Модель генератора не была загружена")]
    GeneratorModelNotLoad,
    #[error("Ошибка загрузки модели `{source}`")]
    EmbeddingModelLoadError
    {
        source: anyhow::Error
    },
    #[error("Ошибка загрузки модели `{model}` -> `{source}`")]
    ModelLoadError
    {
        model: String,
        source: anyhow::Error
    },

    #[error("Ошибка модели `{source}`")]
    ModelError
    {
        source: anyhow::Error
    },
  

}
