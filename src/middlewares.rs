use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{Response, IntoResponse, Redirect},
};
use chrono::Utc;
use cookie::Cookie;
use crate::db::establish_connection;
use crate::session_backend::{delete_expired_sessions, get_session};

// pub async fn auth_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
//     if let Some(auth_header) = req.headers().get("Cookie") {
//         if auth_header.to_str().unwrap().contains("session_id=") {
//             Ok(next.run(req).await)
//         } else {
//             Ok(Redirect::to("/auth/login").into_response())
//         }
//     } else {
//         Ok(Redirect::to("/auth/login").into_response())
//     }
// }


pub async fn session_middleware(mut req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    if let Some(cookie_header) = req.headers().get("Cookie") {
        let cookies_str = cookie_header.to_str().ok().unwrap_or_default();

        let cookies = cookies_str
            .split(';')
            .filter_map(|cookie_str| {
                let parsed = Cookie::parse_encoded(cookie_str.trim());
                if parsed.is_err() {
                    tracing::debug!("Failed to parse cookie: {}", cookie_str);
                }
                parsed.ok()
            })
            .collect::<Vec<_>>();

        if let Some(session_cookie) = cookies.iter().find(|cookie| cookie.name() == "session_id") {
            let session_id = session_cookie.value();

            let conn = &mut establish_connection().await;
            match get_session(conn, session_id).await {
                Ok(Some(session)) => {
                    let current_time = Utc::now().naive_utc();
                    if current_time < session.expires_at {
                        tracing::info!("Valid session for user: {}", session.user_id);
                        req.extensions_mut().insert(session);
                        Ok(next.run(req).await)
                    } else {
                        tracing::warn!("Session expired for ID: {}. Current: {}, Expires: {}", session_id, current_time, session.expires_at);
                        let _ = delete_expired_sessions(conn).await;
                        Ok(Redirect::to("/auth/login").into_response())
                    }
                }
                Ok(None) => {
                    tracing::warn!("Session ID not found in database: {}", session_id);
                    Ok(Redirect::to("/auth/login").into_response())
                }
                Err(e) => {
                    tracing::error!("Database error during session lookup: {}", e);
                    Ok(Redirect::to("/auth/login").into_response())
                }
            }
        } else {
            tracing::info!("Required 'session_id' cookie missing. Found: {:?}", cookies.iter().map(|c| c.name()).collect::<Vec<_>>());
            Ok(Redirect::to("/auth/login").into_response())
        }
    } else {
        tracing::info!("No cookies present in request headers.");
        Ok(Redirect::to("/auth/login").into_response())
    }
}


pub async fn security_headers_middleware(req: Request<Body>, next: Next) -> Response {
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
