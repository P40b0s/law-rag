use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts, response::{IntoResponse, Response}};

use crate::user::User;

// pub struct FingerprintExtractor(pub Arc<String>);
// impl<S> FromRequestParts<S> for FingerprintExtractor
// where
//     S: Send + Sync,
//     Arc<AppState>: FromRef<S>
// {
//     type Rejection = Error;

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> 
//     {
//         // Извлекаем заголовки из запроса
//         let app_state = Arc::<AppState>::from_ref(state);
//         if let Some(fingerprint) = parts.headers.get(&app_state.configuration.fingerprint_header_name)
//         {
//             if let Ok(f) = fingerprint.to_str()
//             {
//                 return Ok(FingerprintExtractor(Arc::new(f.to_string())));
//             }
//         }
//         Err(Error::FingerprintNotFound)
//     }
// }


// impl IntoResponseParts for ResponseSessionWrapper
// {
//     type Error = std::convert::Infallible;
//     fn into_response_parts(self, mut response: ResponseParts) -> Result<ResponseParts, Self::Error> 
//     {
//         // Добавляем cookies в заголовки ответа
//         let headers = response.headers_mut();
//         let cookie = self.to_cookie();
//         let header_value = HeaderValue::from_str(&cookie);
//         if let Ok(hv) = header_value
//         {
//             headers.append("Set-Cookie", hv);
//         }
//         else 
//         {
//             logger::error!("Failed to convert cookie `{}` to header value", cookie);    
//         }
//         Ok(response)
//     }
// }
#[derive(Clone)]
pub struct UserExtension
{
    pub user: Arc<User>,
}
impl<S> FromRequestParts<S> for UserExtension
where
    S: Send + Sync,
{
    type Rejection = Response;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> 
    {
        if let Some(user) = parts.extensions.get::<UserExtension>()
        {
            Ok(user.clone())
        }
        else 
        {
            Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Ошибка экстрактора user").into_response())
        }
    }
}