mod auth;
mod proxy;
mod services;
mod middleware;
mod extensions;
mod static_files_router;

use std::sync::Arc;
use axum::Router;
use crate::api::middleware::TokenAuthLayer;
use crate::state::AppState;

pub fn router(state: Arc<AppState>) -> Router
{
    // Прокси-роутер — все запросы проходят через AuthLayer
    let proxy_router = Router::new()
        .fallback(proxy::proxy_handler)
        .layer(TokenAuthLayer::new(state.clone()))
        .with_state(state.clone());

    // Роутер без AuthLayer: управление шлюзом + встроенный UI
    let open_router = Router::new()
        .nest("/gateway-admin/services", services::services_router(state.clone()))
        .nest("/auth", auth::auth_router(state))
        .nest("/ui", static_files_router::ui_router());

    proxy_router.merge(open_router)
}
