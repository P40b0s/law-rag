use std::{net::SocketAddr, sync::Arc};
use axum::{body::Body, extract::{ConnectInfo, DefaultBodyLimit, Multipart, Path, State}, http::{header, HeaderValue, StatusCode}, response::{IntoResponse, Response}, routing::{get, post}, Extension, Json, Router};
use futures::StreamExt;
use serde::Serialize;
use tokio::{fs::create_dir_all, io::AsyncWriteExt};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use uuid::Uuid;
use crate::{Error, api::IdPayload, state::AppState};
use super::layers::{cors_layer};




pub fn documents_router(app_state: Arc<AppState>) -> Router
{   
    Router::new()      
        .route(&super::with_api_version(super::ApiVersion::V1,"/documents/{dep_id}"), 
        get(get_tasks))
        .with_state(app_state.clone())
        .layer(cors_layer(app_state))
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)))
}


pub async fn get_tasks(
    ConnectInfo(_): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,
    dep_id: Option<Path<Uuid>>)
-> Result<Response<Body>, Error>
{
        // let emp = match dep_id
        // {
        //     Some(Path(id)) =>  app_state.services.tasks_service.get(Some(id)).await?,
        //     None => app_state.services.tasks_service.get(None).await?,
        // };
    Ok((
        StatusCode::OK,
        Json("asdasdasd"),
    ).into_response())
}
