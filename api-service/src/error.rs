use axum::{http::{HeaderMap, StatusCode}, response::{IntoResponse, Response}};
use jwt_authentification::{Cookie, Duration as CookieMaxLife};
use serde::Serialize;
use thiserror::Error;
use utilites::Date;

#[derive(Error, Debug)]
pub enum Error 
{
    #[error(transparent)]
    DeserializeError(#[from] serde_json::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Ошибка авторизации: `{0}`")]
    UtilitesError(#[from] utilites::error::Error),
    #[error("Ошибка, путь не найден")]
    PathNotFound,
    #[error(transparent)]
    PipelineError(#[from] rag_service::Error),
    #[error("Не найдено ни одного чанка в документе `{0}` от `{1}`")]
    ChunksIsEmpty(String, String),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    #[error(transparent)]
    DatabaseError(#[from] database::Error),
    #[error("Ошибка загрузки моделей: `{0}`")]
    ModelsError(String),
    #[error("Уже идет обработка документа: `{0}`")]
    EmbeddingInProgress(String),
}

impl Error
{
    pub fn empty_chunk(number: &str, date: String) -> Self
    {
        Self::ChunksIsEmpty(number.to_owned(), date)
    }
}

impl serde::Serialize for Error 
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
	S: serde::ser::Serializer,
	{
		serializer.serialize_str(self.to_string().as_ref())
	}
}

impl IntoResponse for Error
{
    fn into_response(self) -> axum::response::Response 
    {
        let message = self.to_string();
        match self
        {
            Error::IoError(e) =>
            {
                let body = e.to_string();
                logger::error!("{:?}", &body);
                ServerErrorResponse::new(StatusCode::BAD_REQUEST, body).into_response()
            },
            _ => 
            {
                let body = self.to_string();
                logger::error!("{:?}", &body);
                ServerErrorResponse::new(StatusCode::BAD_REQUEST, body).into_response()
            }
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ServerErrorResponse
{
    err: String
}
impl ServerErrorResponse
{
    pub fn new(status_code: StatusCode, error: String) -> impl IntoResponse
    {
        let e = Self {err: error};
        (status_code, serde_json::to_string(&e).unwrap())
    }
}
