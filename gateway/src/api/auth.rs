use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::{StatusCode, header::SET_COOKIE};
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::pow;
use crate::services::UserDbo;
use crate::state::AppState;

// --- DTO ---

#[derive(Deserialize)]
pub struct LoginRequest
{
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest
{
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub second_name: String,
    pub surname: String,
}

#[derive(Serialize)]
pub struct TokenResponse
{
    pub token: String,
}

#[derive(Serialize)]
pub struct ChallengeResponse
{
    pub challenge: String,
    pub difficulty: usize,
}

#[derive(Deserialize)]
pub struct GuestTokenRequest
{
    pub challenge: String,
    pub nonce: u64,
}

#[derive(Serialize)]
pub struct GuestServiceInfo
{
    pub id: i64,
    pub prefix: String,
    pub service_name: String,
}

// --- Handlers ---

/// `POST /auth/login`
/// Принимает `username` + `password`, возвращает JWT-токен.
async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> impl IntoResponse
{
    let user = match state.users_repository.get_by_username(&body.username).await
    {
        Ok(u) => u,
        Err(Error::UserNotFound) =>
        {
            return (StatusCode::UNAUTHORIZED, "Неверное имя пользователя или пароль").into_response();
        }
        Err(e) => return e.into_response(),
    };

    if user.password != body.password
    {
        return (StatusCode::UNAUTHORIZED, "Неверное имя пользователя или пароль").into_response();
    }

    match state.jwt_service.gen_key(user.id, 60)
    {
        Ok(token) => Json(TokenResponse { token }).into_response(),
        Err(e) => e.into_response(),
    }
}

/// `POST /auth/register`
/// Создаёт нового пользователя. Права назначаются отдельно через `/gateway-admin`.
async fn register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> impl IntoResponse
{
    let dbo = UserDbo
    {
        id: uuid::Uuid::now_v7(),
        username: body.username,
        password: body.password,
        first_name: body.first_name,
        second_name: body.second_name,
        surname: body.surname,
        avatar: None,
        access: vec![],
    };

    match state.users_repository.create(&dbo).await
    {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => e.into_response(),
    }
}

/// `GET /auth/guest-challenge`
/// Генерирует новый PoW-challenge и сохраняет его в памяти (TTL 10 мин).
async fn guest_challenge(State(state): State<Arc<AppState>>) -> impl IntoResponse
{
    let challenge = pow::generate_challenge();

    // Очищаем просроченные challenge попутно
    {
        let ttl = Duration::from_secs(pow::CHALLENGE_TTL_SECS);
        let mut map = state.challenges.lock().unwrap();
        map.retain(|_, created| created.elapsed() < ttl);
        map.insert(challenge.clone(), Instant::now());
    }

    Json(ChallengeResponse
    {
        challenge,
        difficulty: pow::DIFFICULTY,
    })
}

/// `POST /auth/guest-token`
/// Принимает `{ challenge, nonce }`, проверяет PoW, возвращает короткоживущий JWT.
async fn guest_token(
    State(state): State<Arc<AppState>>,
    Json(body): Json<GuestTokenRequest>,
) -> impl IntoResponse
{
    // 1. Проверяем что challenge существует и не просрочен
    let valid = {
        let ttl = Duration::from_secs(pow::CHALLENGE_TTL_SECS);
        let mut map = state.challenges.lock().unwrap();
        if let Some(created) = map.remove(&body.challenge)
        {
            created.elapsed() < ttl
        }
        else
        {
            false
        }
    };

    if !valid
    {
        return (StatusCode::BAD_REQUEST, "Challenge не найден или просрочен").into_response();
    }

    // 2. Проверяем PoW-решение
    if !pow::verify(&body.challenge, body.nonce)
    {
        return (StatusCode::BAD_REQUEST, "Неверное решение PoW").into_response();
    }

    // 3. Выдаём короткоживущий гостевой JWT + cookie для browser navigation
    match state.jwt_service.gen_key("guest", pow::GUEST_TOKEN_LIFETIME_MIN)
    {
        Ok(token) =>
        {
            let max_age = pow::GUEST_TOKEN_LIFETIME_MIN * 60;
            let cookie = format!(
                "guest_token={}; Path=/; SameSite=Strict; Max-Age={}",
                token, max_age
            );
            ([(SET_COOKIE, cookie)], Json(TokenResponse { token })).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// `GET /auth/guest-services`
/// Публичный endpoint — возвращает список сервисов с `allow_guests = true`.
async fn guest_services(State(state): State<Arc<AppState>>) -> impl IntoResponse
{
    match state.services_repository.get_guest_services().await
    {
        Ok(list) =>
        {
            let body: Vec<GuestServiceInfo> = list
                .into_iter()
                .map(|s| GuestServiceInfo
                {
                    id: s.id,
                    prefix: s.prefix,
                    service_name: s.service_name,
                })
                .collect();
            Json(body).into_response()
        }
        Err(e) => e.into_response(),
    }
}

// --- Router ---

pub fn auth_router(state: Arc<AppState>) -> Router
{
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/guest-challenge", get(guest_challenge))
        .route("/guest-token", post(guest_token))
        .route("/guest-services", get(guest_services))
        .with_state(state)
}
