use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use std::env;

pub async fn require_api_key(req: Request, next: Next) -> Result<Response, StatusCode> {
    let api_key = env::var("API_KEY").unwrap_or_else(|_| "".to_string());
    if api_key.is_empty() {
        tracing::error!("API_KEY environment variable is not set");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if let Some(auth_header) = req.headers().get(header::HeaderName::from_static("x-api-key")) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str == api_key {
                return Ok(next.run(req).await);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
