use std::{net::SocketAddr, sync::Arc};
use tracing::debug;
use crate::state::AppState;
use crate::api::router;

pub async fn start() -> Result<(), crate::Error>
{
    let state = Arc::new(AppState::initialize().await?);
    let addr = SocketAddr::from(([0, 0, 0, 0], state.configuration.server_port));
    debug!("Апи сервера доступно на {}", &addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router(state).into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
    Ok(())
}


#[cfg(test)]
mod tests
{
    use crate::logger;

    #[tokio::test]
    async fn test_running()
    {
        logger::init();
        let _ = super::start().await;
        loop 
        {
            tokio::time::sleep(tokio::time::Duration::from_millis(60000)).await;
        }
    }
}