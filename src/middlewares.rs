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
