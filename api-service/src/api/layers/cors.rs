use axum::http::{HeaderValue, Method};//http::{Request, Response, Method, header};
use axum::http::header::{ACCEPT, CONTENT_LENGTH, ACCESS_CONTROL_ALLOW_HEADERS, AUTHORIZATION, CONTENT_TYPE, ORIGIN};
use tower_http::cors::CorsLayer;
use std::sync::Arc;

use crate::state::AppState;

pub fn cors_layer(state: Arc<AppState>) -> CorsLayer
{
    let origins: Vec<HeaderValue> = state.configuration.origins.iter().map(|v| v.parse().unwrap()).collect();
    let cors_layer = CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PUT, Method::HEAD, Method::PATCH])
            .allow_headers([ORIGIN, ACCEPT, CONTENT_TYPE, CONTENT_LENGTH, ACCESS_CONTROL_ALLOW_HEADERS, AUTHORIZATION])
            .allow_credentials(true);
        //"Access-Control-Allow-Headers", "Access-Control-Allow-Headers, Origin,Accept, X-Requested-With, Content-Type, Access-Control-Request-Method, Access-Control-Request-Headers"
            //.allow_headers(vec![AUTHORIZATION, ACCEPT]);
    return cors_layer;
}