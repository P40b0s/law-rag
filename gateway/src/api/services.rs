use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::routing::{delete, get, post, put};
use serde::{Deserialize, Serialize};

use crate::services::ServiceDbo;
use crate::state::AppState;

// --- DTO ---

#[derive(Deserialize)]
pub struct ServiceRequest
{
    pub prefix: String,
    pub service_name: String,
    pub targets: Vec<String>,
    pub permissions: Vec<String>,
    #[serde(default)]
    pub allow_guests: bool,
}

#[derive(Serialize)]
pub struct ServiceResponse
{
    pub id: i64,
    pub prefix: String,
    pub service_name: String,
    pub targets: Vec<String>,
    pub permissions: Vec<String>,
    pub allow_guests: bool,
}

impl From<ServiceDbo> for ServiceResponse
{
    fn from(d: ServiceDbo) -> Self
    {
        Self
        {
            id: d.id,
            prefix: d.prefix,
            service_name: d.service_name,
            targets: d.targets,
            permissions: d.permissions,
            allow_guests: d.allow_guests,
        }
    }
}

// --- Handlers ---

async fn list(State(state): State<Arc<AppState>>) -> impl IntoResponse
{
    match state.services_repository.get_all().await
    {
        Ok(services) =>
        {
            let resp: Vec<ServiceResponse> = services.into_iter().map(Into::into).collect();
            Json(resp).into_response()
        }
        Err(e) => e.into_response(),
    }
}

async fn create(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ServiceRequest>,
) -> impl IntoResponse
{
    let dbo = ServiceDbo
    {
        id: 0,
        prefix: body.prefix,
        service_name: body.service_name,
        targets: body.targets,
        permissions: body.permissions,
        allow_guests: body.allow_guests,
    };
    match state.services_repository.create(&dbo).await
    {
        Ok(id) =>
        {
            let resp = ServiceResponse
            {
                id,
                prefix: dbo.prefix,
                service_name: dbo.service_name,
                targets: dbo.targets,
                permissions: dbo.permissions,
                allow_guests: dbo.allow_guests,
            };
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

async fn update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<ServiceRequest>,
) -> impl IntoResponse
{
    let dbo = ServiceDbo
    {
        id,
        prefix: body.prefix,
        service_name: body.service_name,
        targets: body.targets,
        permissions: body.permissions,
        allow_guests: body.allow_guests,
    };
    match state.services_repository.update(&dbo).await
    {
        Ok(()) => Json(ServiceResponse::from(dbo)).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn remove(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse
{
    match state.services_repository.delete(id).await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}

// --- Router ---

pub fn services_router(state: Arc<AppState>) -> Router
{
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", put(update).delete(remove))
        .with_state(state)
}
