use std::sync::Arc;
use axum::{response::IntoResponse, routing::get, Router};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use crate::{state::AppState, Error};


pub fn router(app_state: Arc<AppState>) -> Router
{   
    let static_router = super::static_router(Arc::clone(&app_state));
    let documents_router = super::documents::documents_router(Arc::clone(&app_state));
    Router::new()      
        .route(&super::with_api_version(super::ApiVersion::V1, "/sse"), get(crate::services::sse_handler))
        .with_state(app_state.clone())
        .layer(super::layers::cors_layer(app_state.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .merge(documents_router)
        .merge(static_router)
}


async fn handler_404() -> impl IntoResponse 
{
    Error::PathNotFound
}