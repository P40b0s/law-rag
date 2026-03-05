use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, Response, StatusCode};
use axum::response::IntoResponse;
use tracing::{debug, error};

use crate::state::AppState;
use crate::api::extensions::UserExtension;

/// Универсальный прокси-обработчик.
///
/// Схема URL: `/{prefix}/{rest_path}`
/// - `prefix` — идентифицирует сервис в конфигурации
/// - `rest_path` — путь, который будет передан на бэкенд
///
/// Выбор бэкенда из `targets` осуществляется по round-robin.
/// Тело, метод, заголовки и query-параметры прозрачно пробрасываются.
pub async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> Response<Body>
{
    let uri = req.uri().clone();
    let path = uri.path();

    // Извлекаем первый непустой сегмент — prefix сервиса
    let prefix = match path.split('/').find(|s| !s.is_empty())
    {
        Some(p) => p,
        None =>
        {
            return (StatusCode::NOT_FOUND, "Не указан префикс сервиса").into_response();
        }
    };

    // Остаток пути после prefix (то что пойдёт на бэкенд)
    let rest = path
        .strip_prefix(&format!("/{}", prefix))
        .unwrap_or("/");
    let rest = if rest.is_empty() { "/" } else { rest };

    // Находим сервис по prefix (из БД)
    let service = match state.services_repository.get_by_prefix(prefix).await
    {
        Ok(Some(s)) => s,
        Ok(None) =>
        {
            error!("Сервис с префиксом `{}` не найден в БД", prefix);
            return (StatusCode::NOT_FOUND, "Сервис не найден").into_response();
        }
        Err(e) =>
        {
            error!("Ошибка запроса сервиса `{}`: {}", prefix, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Ошибка базы данных").into_response();
        }
    };

    if service.targets.is_empty()
    {
        error!("Нет адресов для сервиса `{}`", service.service_name);
        return (StatusCode::BAD_GATEWAY, "Нет доступных адресов сервиса").into_response();
    }

    // Round-robin: счётчик создаётся лениво при первом обращении
    let idx =
    {
        let mut map = state.counters.lock().unwrap();
        let counter = map.entry(prefix.to_string())
            .or_insert_with(|| Arc::new(AtomicUsize::new(0)));
        counter.fetch_add(1, Ordering::Relaxed) % service.targets.len()
    };
    let target = service.targets[idx].trim_end_matches('/');

    // Собираем итоговый URL
    let target_url = match uri.query()
    {
        Some(query) => format!("{}{}?{}", target, rest, query),
        None => format!("{}{}", target, rest),
    };
    debug!("Проксируем {} {} → {}", req.method(), path, target_url);

    // Извлекаем метод, заголовки и user ДО consume body
    let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes())
        .unwrap_or(reqwest::Method::GET);

    // Извлекаем пользователя из extensions (добавлен AuthMiddleware)
    let user_ext = req.extensions().get::<UserExtension>().cloned();

    // Пробрасываем заголовки, пропуская hop-by-hop
    let mut fwd_headers = reqwest::header::HeaderMap::new();
    for (name, value) in req.headers()
    {
        if matches!(
            name.as_str(),
            "host" | "connection" | "transfer-encoding" | "te" | "trailer" | "upgrade"
        )
        {
            continue;
        }
        if let (Ok(n), Ok(v)) = (
            reqwest::header::HeaderName::from_bytes(name.as_ref()),
            reqwest::header::HeaderValue::from_bytes(value.as_bytes()),
        )
        {
            fwd_headers.insert(n, v);
        }
    }

    // Добавляем заголовки с данными пользователя для целевого сервиса
    if let Some(ext) = user_ext
    {
        let user = &ext.user;

        if let Ok(v) = reqwest::header::HeaderValue::from_str(&user.id.to_string())
        {
            fwd_headers.insert("x-user-id", v);
        }

        // Права пользователя для данного конкретного сервиса
        let service_permissions = user.access
            .iter()
            .find(|a| a.service_name == service.service_name)
            .map(|a| a.permissions.join(","))
            .unwrap_or_default();

        if let Ok(v) = reqwest::header::HeaderValue::from_str(&service_permissions)
        {
            fwd_headers.insert("x-user-permissions", v);
        }
    }

    // Читаем тело запроса
    let body_bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await
    {
        Ok(b) => b,
        Err(e) =>
        {
            error!("Ошибка чтения тела запроса: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Ошибка чтения тела запроса").into_response();
        }
    };

    // Отправляем запрос на бэкенд
    let backend_resp = state
        .http_client
        .request(method, &target_url)
        .headers(fwd_headers)
        .body(body_bytes)
        .send()
        .await;

    match backend_resp
    {
        Ok(resp) =>
        {
            let status = StatusCode::from_u16(resp.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            let mut builder = Response::builder().status(status);

            // Пробрасываем заголовки ответа
            for (name, value) in resp.headers()
            {
                if let (Ok(n), Ok(v)) = (
                    axum::http::header::HeaderName::from_bytes(name.as_ref()),
                    axum::http::header::HeaderValue::from_bytes(value.as_bytes()),
                )
                {
                    builder = builder.header(n, v);
                }
            }

            // Стримим тело ответа (работает и для SSE, и для обычных ответов)
            let stream = resp.bytes_stream();
            let body = Body::from_stream(stream);

            builder.body(body).unwrap_or_else(|e|
            {
                error!("Ошибка построения ответа: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })
        }
        Err(e) =>
        {
            error!("Ошибка проксирования на `{}`: {}", target_url, e);
            (StatusCode::BAD_GATEWAY, format!("Ошибка соединения с сервисом: {}", e)).into_response()
        }
    }
}
