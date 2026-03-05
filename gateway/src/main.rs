mod api;
mod error;
mod user;
mod configuration;
mod services;
mod logger;
mod pow;
mod state;

use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main()
{
    logger::init();

    let app_state = Arc::new(
        state::AppState::new().await
            .expect("Ошибка инициализации AppState")
    );

    let port = app_state.configuration.port;
    let addr = format!("0.0.0.0:{}", port);

    let router = api::router(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await
        .expect("Не удалось привязаться к адресу");

    info!("Шлюз запущен на {}", addr);

    axum::serve(listener, router).await
        .expect("Ошибка сервера");
}
