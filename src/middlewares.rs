use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum::response::{IntoResponse, Redirect};

pub async fn auth_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    if let Some(auth_header) = req.headers().get("Cookie") {
        if auth_header.to_str().unwrap().contains("id=") {
            Ok(next.run(req).await)
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    } else {
        Ok(Redirect::to("/auth/login").into_response())
    }
}
