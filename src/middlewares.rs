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
            .filter_map(|cookie_str| Cookie::parse_encoded(cookie_str.trim()).ok())
            .collect::<Vec<_>>();

        if let Some(session_cookie) = cookies.iter().find(|cookie| cookie.name() == "session_id") {
            let session_id = session_cookie.value();

            let conn = &mut establish_connection().await;
            if let Ok(Some(session)) = get_session(conn, session_id) {
                let current_time = Utc::now().naive_utc(); // Current datetime in UTC
                tracing::info!("{:?}", session);
                if current_time < session.expires_at {
                    tracing::info!("Session is valid {} -- {}", current_time, session.expires_at);
                    req.extensions_mut().insert(session);
                    Ok(next.run(req).await)
                } else {
                    tracing::info!("Session expired {} -- {}", current_time, session.expires_at);
                    delete_expired_sessions(conn).expect("Error deleting expired sessions");
                    Ok(Redirect::to("/auth/login").into_response())
                }
            } else {
                tracing::info!("Session not found in DB");
                Ok(Redirect::to("/auth/login").into_response())
            }
        } else {
            tracing::info!("Session cookie not found");
            Ok(Redirect::to("/auth/login").into_response())
        }
    } else {
        tracing::info!("No cookies found");
        Ok(Redirect::to("/auth/login").into_response())
    }
}
