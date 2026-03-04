use std::sync::Arc;
use std::task::{Context, Poll};
use axum::body::Body;
use axum::response::IntoResponse;
use gatehouse::{PermissionChecker, PolicyBuilder};
use tower::{Service, Layer};
use axum::http::{HeaderMap, Request, Response, StatusCode, Uri};
use futures::future::BoxFuture;
use futures::FutureExt;
use tracing::{debug, error};
use utilites::http::AUTHORIZATION;
use crate::configuration::Configuration;
use crate::error::ServerErrorResponse;
use crate::state::AppState;
use crate::user::User;
use super::user_extension::UserExtension;

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
pub struct AuthLayer
{
    state: Arc<AppState>,
}

impl AuthLayer
{
    pub fn new(state: Arc<AppState>) -> Self
    {
        Self { state }
    }
}

impl<S> tower::Layer<S> for AuthLayer
where
    S: Service<Request<axum::body::Body>, Response = Response<axum::body::Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Service = AuthMiddleware<S>;
    fn layer(&self, inner: S) -> Self::Service
    {
        AuthMiddleware::new(inner, self.state.clone())
    }
}

/// Проверяет авторизацию и прокидывает `UserExtension` в extensions запроса
#[derive(Clone)]
pub struct AuthMiddleware<S>
{
    inner: S,
    state: Arc<AppState>,
}

impl<S> AuthMiddleware<S> 
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
impl<S> Service<Request<axum::body::Body>> for AuthMiddleware<S>
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
            let bearer_check = bearer_checker(headers, Arc::clone(&state)).await;
            if let Ok(user) = bearer_check
            {
                if let Err(e) = permissions_checker(req.uri(), &user, &*state.configuration)
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

fn permissions_checker(uri: &Uri, user: &User, configuration: &Configuration) -> Result<(), String>
{
    let path = uri.path();
    // split('/') для "/info/search" даёт ["", "info", "search"] — нужен первый непустой сегмент
    if let Some(prefix) = path.split('/').find(|s| !s.is_empty())
    {
        if let Some(service) = configuration.get_service_by_prefix(prefix)
        {
            if let Some(user_permissions) = user.access.iter().find(|f| f.service_name == service.service_name)
            {
                for required_permission in &service.permissions
                {
                    if user_permissions.permissions.contains(required_permission)
                    {
                        return Ok(());
                    }
                }
                error!("Пользователь `{}` не имеет нужных прав для доступа к сервису `{}`", user.username, service.service_name);
                Err("Недостаточно прав для доступа к сервису".to_owned())
            }
            else
            {
                error!("Пользователь `{}` не имеет прав доступа к сервису `{}`", user.username, service.service_name);
                Err("Пользователь не имеет доступа к этому сервису".to_owned())
            }
        }
        else
        {
            error!("Сервис с префиксом `{}` не найден в конфигурации", prefix);
            Err("Сервис не найден".to_owned())
        }
    }
    else
    {
        error!("Не указан префикс сервиса в URL `{}`", path);
        Err("Не указан префикс сервиса".to_owned())
    }
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