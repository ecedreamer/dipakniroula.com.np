use axum::{
    extract::{State, Request},
    middleware::Next,
    response::{Response, IntoResponse, Redirect},
};
use chrono::Utc;
use cookie::Cookie;
use crate::session_backend::{delete_expired_sessions, get_session};
use crate::state::AppState;

pub async fn session_middleware(
    State(state): State<AppState>,
    req: Request, 
    next: Next
) -> Response {
    let session_id = get_session_id_from_req(&req);
    
    if let Some(sid) = session_id {
        let pool = state.db_pool.clone();
        if let Some(session) = get_session_by_id(pool, sid).await {
            let current_time = Utc::now().naive_utc();
            if current_time < session.expires_at {
                let mut req = req;
                req.extensions_mut().insert(session);
                return next.run(req).await;
            } else {
                if let Ok(mut conn) = state.get_conn().await {
                    let _ = delete_expired_sessions(&mut conn).await;
                }
            }
        }
    }
    
    tracing::info!("Authentication required. Redirecting to login.");
    Redirect::to("/auth/login").into_response()
}

pub async fn optional_session_middleware(
    State(state): State<AppState>,
    req: Request, 
    next: Next
) -> Response {
    let session_id = get_session_id_from_req(&req);
    let mut req = req;

    if let Some(sid) = session_id {
        let pool = state.db_pool.clone();
        if let Some(session) = get_session_by_id(pool, sid).await {
            let current_time = Utc::now().naive_utc();
            if current_time < session.expires_at {
                req.extensions_mut().insert(session);
            }
        }
    }
    next.run(req).await
}

fn get_session_id_from_req(req: &Request) -> Option<String> {
    req.headers()
        .get("Cookie")?
        .to_str().ok()?
        .split(';')
        .filter_map(|s| Cookie::parse_encoded(s.trim()).ok())
        .find(|c| c.name() == "session_id")
        .map(|c| c.value().to_string())
}

async fn get_session_by_id(pool: crate::db::DbPool, session_id: String) -> Option<crate::models::CustomSession> {
    let mut conn = pool.get().await.ok()?;
    get_session(&mut conn, &session_id).await.ok().flatten()
}



pub async fn security_headers_middleware(req: axum::extract::Request, next: Next) -> Response {
    let mut response = next.run(req).await;

    let headers = response.headers_mut();

    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());

    let csp = "default-src 'self'; \
               script-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net https://code.jquery.com https://cdnjs.cloudflare.com https://www.googletagmanager.com https://www.google-analytics.com; \
               style-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://fonts.googleapis.com; \
               font-src 'self' https://fonts.gstatic.com https://cdnjs.cloudflare.com https://cdn.jsdelivr.net; \
               img-src 'self' data: https://www.googletagmanager.com https://www.google-analytics.com; \
               connect-src 'self' https://www.google-analytics.com https://region1.google-analytics.com; \
               frame-src https://www.googletagmanager.com; \
               frame-ancestors 'none'; \
               object-src 'none';";
    headers.insert("Content-Security-Policy", csp.parse().unwrap());

    response
}
