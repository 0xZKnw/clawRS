//! Authentication middleware
//!
//! Validates session tokens on all API requests (except /pair).

use crate::server::ServerState;
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

/// Middleware that checks for a valid session token.
/// Accepts token via:
///   - Authorization: Bearer <token> header  (REST API)
///   - ?token=<token> query string           (WebSocket)
/// Skips validation for the `/pair` endpoint.
pub async fn auth_middleware(
    State(state): State<ServerState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path().to_string();

    // Skip auth for pairing and WebSocket endpoints
    if path == "/pair" || path == "/api/pair" || path.ends_with("/ws") {
        return Ok(next.run(request).await);
    }

    // Try Authorization header first, then query string ?token=
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| {
            // Fallback: check query string for ?token=XXX
            request.uri().query().and_then(|q| {
                q.split('&')
                    .find_map(|param| param.strip_prefix("token="))
                    .map(|t| t.to_string())
            })
        });

    match token {
        Some(t) if state.is_valid_session(&t) => Ok(next.run(request).await),
        _ => {
            tracing::warn!("Unauthorized request to {}", path);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
