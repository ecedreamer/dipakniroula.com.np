use std::env;

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, header},
    response::{Html, IntoResponse, Redirect},
    routing::get,
    Router,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;

use crate::middlewares::user_session_middleware;
use crate::models::CustomSession;
use crate::oauth::models::{NewOAuthUser, OAuthUser, QuizAttemptWithUser};
use crate::session_backend::create_session;
use crate::state::AppState;
use crate::utils::error::AppError;

pub fn oauth_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/user/login", get(login_page))
        .route("/user/auth/google", get(google_auth))
        .route("/user/auth/google/callback", get(google_callback))
        .route(
            "/user/panel",
            get(user_panel)
                .layer(axum::middleware::from_fn_with_state(state.clone(), user_session_middleware)),
        )
        .route(
            "/user/logout",
            get(logout_handler)
                .layer(axum::middleware::from_fn_with_state(state, user_session_middleware)),
        )
}

#[derive(Template)]
#[template(path = "user/login.html")]
struct UserLoginTemplate {
    pub google_auth_url: String,
    pub flash: Option<crate::models::FlashData>,
}

pub async fn login_page() -> impl IntoResponse {
    let client_id = env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
    let redirect_uri = env::var("GOOGLE_REDIRECT_URI")
        .unwrap_or_else(|_| "http://127.0.0.1:8081/user/auth/google/callback".to_string());
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?\
         client_id={}&redirect_uri={}&\
         response_type=code&scope=email%20profile&access_type=online",
        client_id, redirect_uri
    );
    Html(UserLoginTemplate { google_auth_url: auth_url, flash: None }.render().unwrap())
}

pub async fn google_auth() -> impl IntoResponse {
    let client_id = env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
    let redirect_uri = env::var("GOOGLE_REDIRECT_URI")
        .unwrap_or_else(|_| "http://127.0.0.1:8081/user/auth/google/callback".to_string());
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?\
         client_id={}&redirect_uri={}&\
         response_type=code&scope=email%20profile&access_type=online",
        client_id, redirect_uri
    );
    Redirect::to(&auth_url)
}

#[derive(Deserialize)]
pub struct AuthCallback {
    pub code: Option<String>,
    pub error: Option<String>,
}

pub async fn google_callback(
    State(state): State<AppState>,
    Query(params): Query<AuthCallback>,
) -> Result<impl IntoResponse, AppError> {
    if params.error.is_some() {
        return Ok(Redirect::to("/user/login").into_response());
    }
    let code = params.code.ok_or_else(|| AppError::Internal("No auth code".into()))?;

    let client_id = env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| AppError::Internal("GOOGLE_CLIENT_ID not set".into()))?;
    let client_secret = env::var("GOOGLE_CLIENT_SECRET")
        .map_err(|_| AppError::Internal("GOOGLE_CLIENT_SECRET not set".into()))?;
    let redirect_uri = env::var("GOOGLE_REDIRECT_URI")
        .unwrap_or_else(|_| "http://127.0.0.1:8081/user/auth/google/callback".to_string());

    let client = reqwest::Client::new();
    let token_params = [
        ("code", code.as_str()),
        ("client_id", &client_id),
        ("client_secret", &client_secret),
        ("redirect_uri", &redirect_uri),
        ("grant_type", "authorization_code"),
    ];

    let token_resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&token_params)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Token exchange failed: {}", e)))?;

    let token_data: serde_json::Value = token_resp.json().await
        .map_err(|e| AppError::Internal(format!("Token parse failed: {}", e)))?;

    let access_token = token_data["access_token"]
        .as_str().ok_or_else(|| AppError::Internal("No access_token".into()))?
        .to_string();

    let user_resp = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .header("Authorization", format!("Bearer {}", access_token))
        .send().await
        .map_err(|e| AppError::Internal(format!("Userinfo failed: {}", e)))?;

    let user_info: serde_json::Value = user_resp.json().await
        .map_err(|e| AppError::Internal(format!("Userinfo parse failed: {}", e)))?;

    let google_id = user_info["id"].as_str().ok_or_else(|| AppError::Internal("No google id".into()))?;
    let email = user_info["email"].as_str().ok_or_else(|| AppError::Internal("No email".into()))?;
    let name = user_info["name"].as_str().unwrap_or("User");
    let avatar_url = user_info["picture"].as_str();

    let mut conn = state.get_conn().await?;
    let user = find_or_create_user(&mut conn, google_id, email, name, avatar_url).await?;

    let session = create_session(&mut conn, &user.email, "user").await
        .map_err(|e| AppError::Internal(format!("Session error: {}", e)))?;

    let mut headers = HeaderMap::new();
    let cookie_value = format!("session_id={}; HttpOnly; SameSite=Lax; Path=/", session.session_id);
    headers.insert(header::SET_COOKIE, HeaderValue::from_str(&cookie_value).unwrap());

    Ok((headers, Redirect::to("/user/panel")).into_response())
}

async fn find_or_create_user(
    conn: &mut crate::db::PooledConn,
    g_id: &str,
    g_email: &str,
    g_name: &str,
    g_avatar: Option<&str>,
) -> Result<OAuthUser, AppError> {
    use crate::schema::oauth_users::dsl::*;

    if let Some(user) = oauth_users.filter(google_id.eq(g_id))
        .first::<OAuthUser>(conn).await.optional()
        .map_err(|e| AppError::DatabaseError(e))?
    {
        return Ok(user);
    }

    diesel::insert_into(oauth_users).values(&NewOAuthUser {
        google_id: g_id,
        email: g_email,
        name: g_name,
        avatar_url: g_avatar,
    }).execute(conn).await.map_err(|e| AppError::DatabaseError(e))?;

    oauth_users.filter(google_id.eq(g_id))
        .first::<OAuthUser>(conn).await.map_err(|e| AppError::DatabaseError(e))
}

#[derive(Template)]
#[template(path = "user/panel.html")]
struct UserPanelTemplate {
    user: OAuthUser,
    attempts: Vec<QuizAttemptWithUser>,
    average_score: i32,
    total_correct: i32,
    total_questions: i32,
    flash: Option<crate::models::FlashData>,
}

pub async fn user_panel(
    State(state): State<AppState>,
    Extension(session): Extension<CustomSession>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;

    let user = crate::schema::oauth_users::dsl::oauth_users
        .filter(crate::schema::oauth_users::dsl::email.eq(&session.user_id))
        .first::<OAuthUser>(&mut conn).await
        .map_err(|_| AppError::NotFound("User not found".into()))?;

    let attempts = crate::schema::quiz_attempts::dsl::quiz_attempts
        .filter(crate::schema::quiz_attempts::dsl::oauth_user_id.eq(user.id))
        .order(crate::schema::quiz_attempts::dsl::played_at.desc())
        .load::<QuizAttemptWithUser>(&mut conn).await
        .map_err(|e| AppError::DatabaseError(e))?;

    let total_correct: i32 = attempts.iter().map(|a| a.score).sum();
    let total_questions: i32 = attempts.iter().map(|a| a.total_questions).sum();
    let average_score = if total_questions > 0 {
        (total_correct * 100 / total_questions) as i32
    } else { 0 };

    Ok(Html(UserPanelTemplate {
        user, attempts, average_score, total_correct, total_questions, flash: None,
    }.render()?))
}

pub async fn logout_handler(
    State(state): State<AppState>,
    Extension(session): Extension<CustomSession>,
) -> impl IntoResponse {
    if let Some(sess_id) = session.id {
        if let Ok(mut conn) = state.get_conn().await {
            use crate::schema::sessions::dsl::*;
            let _ = diesel::delete(sessions.filter(id.eq(sess_id))).execute(&mut conn).await;
        }
    }
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, HeaderValue::from_str("session_id=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0").unwrap());
    (headers, Redirect::to("/user/login"))
}
