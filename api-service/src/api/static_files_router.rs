use std::{path::PathBuf, sync::Arc};

use axum::{Router, body::Body, extract::State, http::{StatusCode, Uri, header}, response::{IntoResponse, Response}};
use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;
use regex::Regex;



    static LOCALHOST_REGEX: Lazy<Regex> = Lazy::new(||Regex::new(
        r#"http://localhost:8081"#
    ).unwrap());

use crate::{configuration::Configuration, services::get_local_ip, state::AppState};

// Включаем собранные файлы фронтенда
static FRONTEND_DIR: Dir = include_dir!("./frontend/dist");

pub fn static_router(app_state: Arc<AppState>) -> Router
{  
    Router::new()
        .fallback(static_or_spa_handler)
        .with_state(app_state)
}

async fn static_or_spa_handler(
    State(state): State<Arc<AppState>>,
    uri: Uri,
) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    
    // Если запрашивается корень или файл не найден - отдаём index.html (SPA)
    if path.is_empty() || !static_file_exists(path) {
        return serve_index_html();
    }
    
    // Пытаемся отдать статический файл
    match serve_static_file(path, state.configuration.clone()) {
        Ok(response) => response,
        Err(_) => serve_index_html(), // Fallback на SPA
    }
}

// Проверка существования статического файла
fn static_file_exists(path: &str) -> bool {
    FRONTEND_DIR.get_entry(path).is_some()
}

// Отдача статического файла
fn serve_static_file(path: &str, config: Arc<Configuration>) -> Result<Response<Body>, StatusCode> {
    let file = FRONTEND_DIR.get_file(path).ok_or(StatusCode::NOT_FOUND)?;
    let mut is_js = false;
    // Определяем Content-Type по расширению
    let content_type = match PathBuf::from(path).extension()
        .and_then(|ext| ext.to_str()) {
        Some("html") => "text/html",
        Some("js") => 
        {
            is_js = true;
            "application/javascript"
        },
        Some("css") => "text/css",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    };
    //dynamicaly change licalhost on frontend to current server addresse
    let body = if is_js
    {
        if let Some(content) = file.contents_utf8() && let Some(addr) = get_local_ip()
        {
            let new_content = LOCALHOST_REGEX.replace(content, format!("http://{}:{}", addr.to_string(), config.server_port.to_string()));
            let new_content = new_content.as_bytes().to_owned();
            Body::from(new_content)
        }
        else 
        {
            Body::from(file.contents())
        }
    }
    else 
    {
        Body::from(file.contents())
    };
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "public, max-age=31536000")
        .body(body)
        .unwrap())
}

// Отдача index.html для SPA
fn serve_index_html() -> Response<Body> {
    let index_html = FRONTEND_DIR
        .get_file("index.html")
        .expect("index.html not found in dist directory");
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .body(Body::from(index_html.contents()))
        .unwrap()
}