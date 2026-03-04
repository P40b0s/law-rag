mod proxy;

use std::sync::Arc;
use axum::Router;
use crate::middleware::AuthLayer;
use crate::state::AppState;

pub fn router(state: Arc<AppState>) -> Router
{
    Router::new()
        // Все запросы идут в proxy_handler — он сам определяет сервис по prefix
        .fallback(proxy::proxy_handler)
        .layer(AuthLayer::new(state.clone()))
        .with_state(state)
}
