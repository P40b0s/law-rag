/// Встроенный gateway-ui.
///
/// Все файлы из `../gateway-ui/dist` компилируются в бинарь.
/// Маршрутизатор монтируется в `/ui` — любой запрос к `/ui/*`
/// обрабатывается здесь.
///
/// Логика:
///   - если файл существует → отдать с правильным Content-Type
///   - иначе → SPA-fallback: отдать `index.html`

use std::path::PathBuf;

use axum::{
    Router,
    body::Body,
    extract::OriginalUri,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use include_dir::{Dir, include_dir};

static UI_DIR: Dir = include_dir!("./gateway-ui/dist");

pub fn ui_router() -> Router
{
    Router::new().fallback(ui_handler)
}

async fn ui_handler(OriginalUri(uri): OriginalUri) -> Response<Body>
{
    // Убираем ведущий `/ui` (nest уже снял его, но OriginalUri сохраняет оригинал)
    let raw = uri.path();
    let path = raw
        .strip_prefix("/ui/")
        .or_else(|| raw.strip_prefix("/ui"))
        .unwrap_or(raw)
        .trim_start_matches('/');

    // Файл есть в dist → отдаём его
    if !path.is_empty()
    {
        if let Some(file) = UI_DIR.get_file(path)
        {
            let content_type = content_type_for(path);
            let cache = if path.contains(".[") || path.ends_with(".js") || path.ends_with(".css")
            {
                "public, max-age=31536000, immutable"
            }
            else
            {
                "no-cache"
            };
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CACHE_CONTROL, cache)
                .body(Body::from(file.contents()))
                .unwrap();
        }
    }

    // Всё остальное → SPA (index.html)
    serve_index()
}

fn serve_index() -> Response<Body>
{
    let file = UI_DIR
        .get_file("index.html")
        .expect("gateway-ui/dist/index.html не найден. Выполните: cd gateway-ui && pnpm build");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(file.contents()))
        .unwrap()
}

fn content_type_for(path: &str) -> &'static str
{
    match PathBuf::from(path).extension().and_then(|e| e.to_str())
    {
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}
