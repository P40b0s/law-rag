use std::{net::SocketAddr, sync::Arc};
use axum::{body::Body, extract::{ConnectInfo, DefaultBodyLimit, Multipart, Path, State}, http::{header, HeaderValue, StatusCode}, response::{IntoResponse, Response}, routing::{get, post}, Extension, Json, Router};
use futures::StreamExt;
use serde::Serialize;
use tokio::{fs::create_dir_all, io::AsyncWriteExt};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use uuid::Uuid;
use crate::{Error, api::{DocumentRequest, GenerationRequest, IdPayload}, state::AppState};
use super::layers::{cors_layer};



 //.route(&super::with_api_version(super::ApiVersion::V1,"/models/{dep_id}"), 
pub fn documents_router(app_state: Arc<AppState>) -> Router
{   
    Router::new()    
    //модели  
        .route(&super::with_api_version(super::ApiVersion::V1,"/models/load_generation_model"), 
        get(load_generation_model))

        .route(&super::with_api_version(super::ApiVersion::V1,"/models/unload_generation_model"), 
        get(unload_generation_model))

        .route(&super::with_api_version(super::ApiVersion::V1,"/models/load_embedding_model"), 
        get(load_embedding_model))

        .route(&super::with_api_version(super::ApiVersion::V1,"/models/unload_embedding_model"), 
        get(unload_embedding_model))

        .route(&super::with_api_version(super::ApiVersion::V1,"/models/get_models_state"), 
        get(get_models_state))
    //документы
        .route(&super::with_api_version(super::ApiVersion::V1,"/documents/request_document"), 
        post(request_document))

    .route(&super::with_api_version(super::ApiVersion::V1,"/documents/{offset}"), 
        get(get_documents))

        .route(&super::with_api_version(super::ApiVersion::V1,"/documents/embedding_document/{hash}"), 
        get(embedding_document))

        .route(&super::with_api_version(super::ApiVersion::V1,"/health_check"), 
        get(health_check))

        .route(&super::with_api_version(super::ApiVersion::V1,"/documents/generation_request"), 
        post(generation_request))

        .route(&super::with_api_version(super::ApiVersion::V1,"/documents/delete_document/{hash}"), 
        get(delete_document))
    
        .with_state(app_state.clone())
        .layer(cors_layer(app_state))
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)))
}


pub async fn load_generation_model(
    ConnectInfo(_): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,)
-> Result<Response<Body>, Error>
{
    let service = app_state.get_services();
    let models_state = service.documents_service.load_generator_model().await?;
    Ok((
        StatusCode::OK,
        Json(models_state),
    ).into_response())
}
pub async fn delete_document(
    ConnectInfo(_): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,
    Path(hash): Path<String>)
-> Result<Response<Body>, Error>
{
    let service = app_state.get_services();
    let models_state = service.documents_service.delete_document(&hash).await?;
    Ok((
        StatusCode::OK,
    ).into_response())
}
pub async fn unload_generation_model(
    ConnectInfo(_): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,)
-> Result<Response<Body>, Error>
{
    let service = app_state.get_services();
    let models_state = service.documents_service.unload_generator_model().await?;
    Ok((
        StatusCode::OK,
        Json(models_state),
    ).into_response())
}

pub async fn load_embedding_model(
    ConnectInfo(_): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,)
-> Result<Response<Body>, Error>
{
    let service = app_state.get_services();
    let models_state = service.documents_service.load_embedding_model().await?;
    Ok((
        StatusCode::OK,
        Json(models_state),
    ).into_response())
}

pub async fn unload_embedding_model(
    ConnectInfo(_): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,)
-> Result<Response<Body>, Error>
{
    let service = app_state.get_services();
    let models_state = service.documents_service.unload_embedding_model().await?;
    Ok((
        StatusCode::OK,
        Json(models_state),
    ).into_response())
}

pub async fn get_models_state(
    ConnectInfo(_): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,)
-> Result<Response<Body>, Error>
{
    let service = app_state.get_services();
    let models_state = service.rag_service.models_state().await;
    Ok((
        StatusCode::OK,
        Json(models_state),
    ).into_response())
}

pub async fn request_document(
    ConnectInfo(_): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<DocumentRequest>)
-> Result<Response<Body>, Error>
{
    let service = app_state.get_services();
    let doc = service.documents_service.get_document_and_add_to_db(req.sign_date, &req.number).await?;
    Ok((
        StatusCode::OK,
        Json(doc),
    ).into_response())
}
pub async fn health_check(
    ConnectInfo(_): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,)
-> Result<Response<Body>, Error>
{
    Ok((
        StatusCode::OK,
    ).into_response())
}

pub async fn get_documents(
    ConnectInfo(_): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,
    Path(offset): Path<u32>)
-> Result<Response<Body>, Error>
{
    let service = app_state.get_services();
    let docs = service.documents_service.get_documents_list(offset, 30).await?;
    Ok((
        StatusCode::OK,
        Json(docs),
    ).into_response())
}
pub async fn embedding_document(
    ConnectInfo(_): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,
    Path(hash): Path<String>)
-> Result<Response<Body>, Error>
{
    let service = app_state.get_services();
    let _ = service.documents_service.embedding_document_from_sqlite(&hash).await?;
    Ok((
        StatusCode::OK,
    ).into_response())
}

pub async fn generation_request(
    ConnectInfo(_): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,
    Json(query): Json<GenerationRequest>)
-> Result<Response<Body>, Error>
{
    let service = app_state.get_services();
    let search_result = service.documents_service.search_context(&query.query, query.limit, query.reranker_limit).await?;
    let _ = service.documents_service.generate_result(&query.query, search_result).await?;
    Ok((
        StatusCode::OK,
    ).into_response())
}