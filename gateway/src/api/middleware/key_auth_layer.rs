/// KeyAuthLayer — авторизация гостевых запросов через заголовок `X-Key`.
///
/// Схема:
/// 1. Клиент получает PoW-challenge (`GET /auth/guest-challenge`)
/// 2. Решает задачу и обменивает решение на короткоживущий JWT (`POST /auth/guest-token`)
/// 3. Отправляет запрос с `X-Key: <guest_jwt>`
///
/// Слой:
/// - Проверяет наличие и валидность JWT в `X-Key`
/// - Убеждается что subject == "guest" (отличие от пользовательского токена)
/// - Проверяет что целевой сервис разрешает гостевой доступ (`allow_guests = true`)
/// - Пропускает запрос без `UserExtension` (proxy handler обрабатывает `None`)

use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::response::IntoResponse;
use tower::{Service, Layer};
use axum::http::{Request, Response, StatusCode, Uri};
use futures::future::BoxFuture;
use futures::FutureExt;
use tracing::error;

use crate::error::ServerErrorResponse;
use crate::services::ServicesRepository;
use crate::state::AppState;

#[derive(Clone)]
pub struct KeyAuthLayer
{
    state: Arc<AppState>,
}

impl KeyAuthLayer
{
    pub fn new(state: Arc<AppState>) -> Self
    {
        Self { state }
    }
}

impl<S> Layer<S> for KeyAuthLayer
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Service = KeyAuthMiddleware<S>;
    fn layer(&self, inner: S) -> Self::Service
    {
        KeyAuthMiddleware::new(inner, self.state.clone())
    }
}

#[derive(Clone)]
pub struct KeyAuthMiddleware<S>
{
    inner: S,
    state: Arc<AppState>,
}

impl<S> KeyAuthMiddleware<S>
{
    pub fn new(inner: S, state: Arc<AppState>) -> Self
    {
        Self { inner, state }
    }
}

impl<S> Service<Request<Body>> for KeyAuthMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>
    {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future
    {
        let state = self.state.clone();
        let mut inner = self.inner.clone();
        async move
        {
            // 1. Читаем заголовок X-Key
            let token_str = match req.headers().get("X-Key").and_then(|v| v.to_str().ok())
            {
                Some(t) => t.to_owned(),
                None =>
                {
                    return Ok(error_response("Отсутствует заголовок X-Key"));
                }
            };

            // 2. Валидируем JWT
            let claims = match state.jwt_service.validate(&token_str).await
            {
                Ok(c) => c,
                Err(e) =>
                {
                    return Ok(error_response(e));
                }
            };

            // 3. Проверяем что это гостевой токен (subject == "guest")
            if claims.user_id() != "guest"
            {
                return Ok(error_response("Заголовок X-Key должен содержать гостевой токен"));
            }

            // 4. Проверяем что сервис разрешает гостевой доступ
            if let Err(resp) = guest_allowed(req.uri(), &state.services_repository).await
            {
                return Ok(resp);
            }

            inner.call(req).await
        }
        .boxed()
    }
}

/// Проверяет флаг `allow_guests` у целевого сервиса.
async fn guest_allowed(uri: &Uri, repo: &ServicesRepository) -> Result<(), Response<Body>>
{
    let prefix = uri.path()
        .split('/')
        .find(|s| !s.is_empty())
        .ok_or_else(|| error_response("Не указан префикс сервиса"))?;

    let service = repo.get_by_prefix(prefix).await
        .map_err(|e| error_response(e))?
        .ok_or_else(||
        {
            error!("Сервис с префиксом `{}` не найден", prefix);
            error_response("Сервис не найден")
        })?;

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

fn error_response<T: ToString>(msg: T) -> Response<Body>
{
    let err = msg.to_string();
    error!("{}", &err);
    ServerErrorResponse::new(StatusCode::UNAUTHORIZED, err).into_response()
}
