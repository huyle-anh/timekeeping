use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::instrument;

use crate::errors::AppError;
use crate::AppState;

/// JWT claims for admin authentication.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // username
    pub role: String,
    pub exp: usize,
}

/// Login request body.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response body.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub message: String,
}

/// Create a JWT token signed with the provided secret.
#[instrument(skip(secret))]
pub fn create_token(username: &str, role: &str, secret: &str) -> Result<String, AppError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::Internal(format!("Time error: {}", e)))?
        .as_secs() as usize;

    let claims = Claims {
        sub: username.to_string(),
        role: role.to_string(),
        exp: now + 86400, // 24 hours
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))
}

/// Verify a JWT token and return the claims.
#[instrument(skip(token, secret))]
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Validation(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims)
}

/// Login handler.
#[instrument(skip(state))]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.username != state.config.admin_username
        || payload.password != state.config.admin_password
    {
        return Err(AppError::Validation("Invalid username or password".to_string()));
    }

    let token = create_token(&payload.username, "Admin", &state.config.jwt_secret)?;

    tracing::info!(username = %payload.username, "Login successful");

    Ok(Json(LoginResponse {
        token,
        message: "Login successful".to_string(),
    }))
}

/// Middleware that checks for a valid JWT token in the Authorization header.
/// Uses `from_fn_with_state` so it can access the JWT secret from AppState.
pub async fn verify_admin_middleware(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next<Body>,
) -> Result<impl IntoResponse, Response> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            AppError::Validation("Missing Authorization header".to_string()).into_response()
        })?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            AppError::Validation("Invalid Authorization header format".to_string()).into_response()
        })?;

    let claims = verify_token(token, &state.config.jwt_secret)
        .map_err(|e| e.into_response())?;

    // Enforce Admin role
    if claims.role != "Admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Forbidden: Admin role required"})),
        )
            .into_response());
    }

    Ok(next.run(request).await)
}
