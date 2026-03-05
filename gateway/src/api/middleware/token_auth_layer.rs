use std::sync::Arc;
use std::task::{Context, Poll};
use axum::body::Body;
use axum::response::IntoResponse;
use gatehouse::{PermissionChecker, PolicyBuilder};
use tower::{Service, Layer};
use axum::http::{HeaderMap, Request, Response, StatusCode, Uri};
use futures::future::BoxFuture;
use futures::FutureExt;
use cookie::Cookie;
use tracing::{debug, error};
use utilites::http::AUTHORIZATION;
use crate::error::ServerErrorResponse;
use crate::services::ServicesRepository;
use crate::state::AppState;
use crate::user::User;
use crate::api::extensions::UserExtension;

// /// Слой обработки маршрута с авторизацией
// #[derive(Clone)]
// pub struct AuthLayer 
// {
//     state: Arc<AppState>,
//     roles: Arc<Vec<Role>>,
//     permissions: Arc<Vec<Permission>>,
// }

// impl AuthLayer 
// {
//     pub fn new(state: Arc<AppState>) -> Self
//     {
//         Self 
//         {
//             state,
//             roles: Arc::new(Vec::new()),
//             permissions: Arc::new(Vec::new()),
//         }
//     }
//     pub fn with_roles(mut self, roles: &[Role]) -> Self 
//     {
//         self.roles = Arc::new(roles.to_vec());
//         self
        
//     }
//     pub fn with_permissions(mut self, permissions: &[Permission]) -> Self 
//     {
//         self.permissions = Arc::new(permissions.to_vec());
//         self
//     }
// }

// impl<S> Layer<S> for AuthLayer
// where
//     S: Service<Request<axum::body::Body>, Response = Response<axum::body::Body>> + Clone + Send + 'static,
//     S::Future: Send + 'static,
// {
//     type Service = AuthMiddleware<S>;
//     fn layer(&self, inner: S) -> Self::Service 
//     {
//         AuthMiddleware::new(inner, self.state.clone(), self.roles.clone(), self.permissions.clone())
//     }
// }

/// Tower Layer — оборачивает роутер в `AuthMiddleware`
#[derive(Clone)]
pub struct TokenAuthLayer
{
    state: Arc<AppState>,
}

impl TokenAuthLayer
{
    pub fn new(state: Arc<AppState>) -> Self
    {
        Self { state }
    }
}

impl<S> tower::Layer<S> for TokenAuthLayer
where
    S: Service<Request<axum::body::Body>, Response = Response<axum::body::Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Service = TokenAuthMiddleware<S>;
    fn layer(&self, inner: S) -> Self::Service
    {
        TokenAuthMiddleware::new(inner, self.state.clone())
    }
}

/// Проверяет авторизацию и прокидывает `UserExtension` в extensions запроса
#[derive(Clone)]
pub struct TokenAuthMiddleware<S>
{
    inner: S,
    state: Arc<AppState>,
}

impl<S> TokenAuthMiddleware<S> 
{
    pub fn new(inner: S, state: Arc<AppState>) -> Self 
    {
        Self 
        {
            inner,
            state,
        }
    }
}

/// Реализация трейта `Service` для middleware
impl<S> Service<Request<axum::body::Body>> for TokenAuthMiddleware<S>
where
    S: Service<Request<axum::body::Body>, Response = Response<axum::body::Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<axum::body::Body>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> 
    {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<axum::body::Body>) -> Self::Future
    {
        let state = self.state.clone();
        let mut inner = self.inner.clone();
        async move
        {
            let headers = req.headers();

            // Гостевой путь: X-Key (fetch/XHR) или cookie guest_token (browser navigation)
            if let Some(guest_jwt) = extract_guest_token(headers)
            {
                return match guest_checker(&guest_jwt, req.uri(), Arc::clone(&state)).await
                {
                    Ok(()) => inner.call(req).await,
                    Err(resp) => Ok(resp),
                };
            }

            // Пользовательский путь: Authorization: Bearer <jwt>
            let bearer_check = bearer_checker(headers, Arc::clone(&state)).await;
            if let Ok(user) = bearer_check
            {
                if let Err(e) = permissions_checker(req.uri(), &user, &state.services_repository).await
                {
                    return Ok(access_denied_response(e));
                }
                else
                {
                    let user_extension = UserExtension
                    {
                        user: Arc::new(user)
                    };
                    let ext = req.extensions_mut();
                    ext.insert(user_extension);
                    inner.call(req).await
                }
            }
            else
            {
                Ok(bearer_check.err().unwrap())
            }
        }
        .boxed()
    }
}

async fn permissions_checker(uri: &Uri, user: &User, repo: &ServicesRepository) -> Result<(), String>
{
    let path = uri.path();
    let prefix = match path.split('/').find(|s| !s.is_empty())
    {
        Some(p) => p,
        None =>
        {
            error!("Не указан префикс сервиса в URL `{}`", path);
            return Err("Не указан префикс сервиса".to_owned());
        }
    };

    let service = repo.get_by_prefix(prefix).await
        .map_err(|e| e.to_string())?;

    let service = match service
    {
        Some(s) => s,
        None =>
        {
            error!("Сервис с префиксом `{}` не найден в БД", prefix);
            return Err("Сервис не найден".to_owned());
        }
    };

    let user_access = match user.access.iter().find(|a| a.service_name == service.service_name)
    {
        Some(a) => a,
        None =>
        {
            error!("Пользователь `{}` не имеет доступа к сервису `{}`", user.username, service.service_name);
            return Err("Пользователь не имеет доступа к этому сервису".to_owned());
        }
    };

    for required in &service.permissions
    {
        if user_access.permissions.contains(required)
        {
            return Ok(());
        }
    }

    error!("Пользователь `{}` не имеет нужных прав для сервиса `{}`", user.username, service.service_name);
    Err("Недостаточно прав для доступа к сервису".to_owned())
}

fn error_response<T: ToString>(body: T) -> Response<axum::body::Body>
{
    let err = body.to_string();
    error!("{}", &err);
    ServerErrorResponse::new(StatusCode::UNAUTHORIZED, err).into_response()
    
}
fn access_denied_response(err: String) -> Response<axum::body::Body>
{
    ServerErrorResponse::new(StatusCode::FORBIDDEN, err).into_response()
    
}


/// Извлекает гостевой токен из заголовка `X-Key` (приоритет) или из cookie `guest_token`.
fn extract_guest_token(headers: &HeaderMap) -> Option<String>
{
    if let Some(v) = headers.get("X-Key").and_then(|v| v.to_str().ok())
    {
        return Some(v.to_owned());
    }
    // Fallback: cookie guest_token (browser navigation)
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    for part in cookie_header.split(';')
    {
        if let Ok(c) = Cookie::parse(part.trim().to_owned())
        {
            if c.name() == "guest_token"
            {
                return Some(c.value().to_owned());
            }
        }
    }
    None
}

/// Проверяет гостевой JWT.
/// Возвращает `Ok(())` если токен валиден и сервис разрешает гостевой доступ.
async fn guest_checker(
    token_str: &str,
    uri: &Uri,
    state: Arc<AppState>,
) -> Result<(), Response<Body>>
{
    let claims = state.jwt_service.validate(token_str).await
        .map_err(|e| error_response(e))?;

    if claims.user_id() != "guest"
    {
        return Err(error_response("Токен не является гостевым"));
    }

    // Проверяем что сервис разрешает гостевой доступ
    let prefix = uri.path()
        .split('/')
        .find(|s| !s.is_empty())
        .ok_or_else(|| error_response("Не указан префикс сервиса"))?;

    let service = state.services_repository.get_by_prefix(prefix).await
        .map_err(|e| error_response(e))?
        .ok_or_else(|| error_response("Сервис не найден"))?;

    if !service.allow_guests
    {
        error!("Сервис `{}` не разрешает гостевой доступ", service.service_name);
        return Err(ServerErrorResponse::new(
            StatusCode::FORBIDDEN,
            "Сервис не разрешает гостевой доступ".to_owned(),
        ).into_response());
    }

    Ok(())
}

async fn bearer_checker(headers: &HeaderMap, state: Arc<AppState>) -> Result<User, Response<Body>>
{
    if let Some(authorization) = headers.get(AUTHORIZATION)
    {
        //get key after Bearer 
        if let Ok(token_str) = authorization.to_str()
        {
            if token_str.len() < 10
            {
                let response = error_response("Bearer не распознан");
                Err(response)
            }
            else
            {
                //cut Bearer
                let token_str = &token_str[7..];
                debug!("input token `{}`", token_str);
                let user_claims = state.jwt_service.validate(token_str).await;
                if let Ok(claims) = user_claims 
                {
                    let user = state.users_repository.get_by_id(claims.user_id().parse().unwrap()).await;
                    if user.is_err()
                    {
                        Err(user.err().unwrap().into_response())
                    }
                    else 
                    {
                        Ok(user.unwrap().into())
                    }
                }
                else
                {
                    let response = error_response(user_claims.err().unwrap());
                    Err(response)
                }
            }
        }
        else
        {
            let response = error_response("Ошибка авторизации, заголовок Authorization имеет ошибки в кодировке");
            Err(response)
        }
    }
    else
    {
        let response = error_response("Ошибка авторизации, отсуствует заголовок Authorization");
        Err(response)
    }
}