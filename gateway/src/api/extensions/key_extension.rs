use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts, response::{IntoResponse, Response}};

use crate::user::User;

#[derive(Clone)]
pub struct KeyExtension
{
    pub user: Arc<String>,
}
impl<S> FromRequestParts<S> for KeyExtension
where
    S: Send + Sync,
{
    type Rejection = Response;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> 
    {
        if let Some(user) = parts.extensions.get::<KeyExtension>()
        {
            Ok(user.clone())
        }
        else 
        {
            Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Ошибка экстрактора key").into_response())
        }
    }
}