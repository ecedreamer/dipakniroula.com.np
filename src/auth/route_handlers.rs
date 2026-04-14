use argon2::{Argon2, PasswordHash, PasswordVerifier};
use serde::Deserialize;

use diesel::prelude::*;

use crate::auth::models::{NewSocialLink, SocialLink, UpdateSocialLink};
use crate::middlewares::session_middleware;
use crate::models::{AdminUser, ContactMessage, CustomSession};
use crate::session_backend::create_session;
use crate::utils::error::AppError;
use crate::state::AppState;
use askama::Template;
use axum::{
    Extension, Form, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, header},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use axum_csrf::CsrfToken;
use diesel_async::RunQueryDsl;

pub async fn auth_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/login", get(login_page))
        .route("/login", post(login_handler))
        .route("/logout", get(logout_handler))
        .route(
            "/admin-panel",
            get(admin_home_page).layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
        .route(
            "/add-social-link",
            get(social_link_create_page)
                .post(social_link_create_handler)
                .layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
        .route(
            "/update-social-link/{data_id}",
            get(social_link_update_page)
                .post(social_link_update_handler)
                .layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
        .route(
            "/delete-social-link/{data_id}",
            get(social_link_delete_handler).layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
        .route(
            "/delete-message/{message_id}",
            get(message_delete_handler).layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
}



#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    pub authenticity_token: String,
}

pub async fn login_page(token: CsrfToken) -> Result<impl IntoResponse, AppError> {
    let context = LoginTemplate {
        authenticity_token: token.authenticity_token().map_err(|e| AppError::Internal(format!("CSRF Error: {}", e)))?,
    };

    Ok((token, Html(context.render()?)))
}

#[derive(Deserialize, Debug)]
pub struct LoginForm {
    authenticity_token: String,
    email: String,
    password: String,
}

pub async fn login_handler(
    State(state): State<AppState>,
    token: CsrfToken,
    Form(form_data): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    if token.verify(&form_data.authenticity_token).is_err() {
        tracing::error!("Suspicious Payload: {}", &form_data.authenticity_token);
        return Ok(Redirect::to("/auth/login").into_response());
    }

    let mut conn: crate::db::PooledConn = state.get_conn().await?;

    use crate::schema::admin_users::dsl::*;

    let result = admin_users
        .filter(email.eq(&form_data.email))
        .limit(1)
        .first::<AdminUser>(&mut conn)
        .await;

    match result {
        Ok(admin_user) => {
            let parsed_hash = PasswordHash::new(&admin_user.password).map_err(|e| AppError::Internal(format!("Hash Error: {}", e)))?;
            match Argon2::default().verify_password(form_data.password.as_bytes(), &parsed_hash) {
                Ok(_) => {
                    let session_obj = create_session(&mut conn, admin_user.email)
                        .await
                        .map_err(|e| AppError::Internal(format!("Session Creation Error: {}", e)))?;

                    let mut headers = HeaderMap::new();
                    let cookie_value = format!(
                        "session_id={}; HttpOnly; SameSite=Lax; Path=/",
                        session_obj.session_id
                    );
                    headers.insert(
                        header::SET_COOKIE,
                        HeaderValue::from_str(&cookie_value).map_err(|e| AppError::Internal(e.to_string()))?,
                    );

                    tracing::info!("Successfully logged in...");
                    let redirect_response = Redirect::to("/auth/admin-panel");
                    Ok((headers, redirect_response).into_response())
                }
                Err(_) => {
                    tracing::error!("Invalid credentials...");
                    Ok(Redirect::to("/auth/login").into_response())
                }
            }
        }
        Err(e) => {
            tracing::error!("Invalid credentials; Error: {}", e);
            Ok(Redirect::to("/auth/login").into_response())
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Pagination {
    pub page: Option<i64>,
    pub query: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/admin_home.html")]
struct AdminHomeTemplate {
    messages: Vec<ContactMessage>,
    blog_count: i64,
    experience_count: i64,
    message_count: i64,
    social_count: i64,
    active_nav: String,
    current_page: i64,
    total_pages: i64,
    pages: Vec<i64>,
    search_query: Option<String>,
}

pub async fn admin_home_page(
    State(state): State<AppState>,
    Extension(_session): Extension<CustomSession>,
    Query(pagination): Query<Pagination>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;

    let page = pagination.page.unwrap_or(1).max(1);
    let per_page = 10;
    let offset = (page - 1) * per_page;

    use crate::schema::blogs::dsl::blogs;
    use crate::schema::experiences::dsl::experiences;
    use crate::schema::messages::dsl::*;
    use crate::schema::social_links::dsl::social_links;

    let mut messages_query = messages.into_boxed();
    let mut count_query = messages.into_boxed();

    if let Some(ref q) = pagination.query {
        if !q.trim().is_empty() {
            let search_term = format!("%{}%", q);
            messages_query = messages_query.filter(
                full_name.ilike(search_term.clone())
                    .or(email.ilike(search_term.clone()))
                    .or(subject.ilike(search_term.clone()))
                    .or(message.ilike(search_term.clone()))
            );
            count_query = count_query.filter(
                full_name.ilike(search_term.clone())
                    .or(email.ilike(search_term.clone()))
                    .or(subject.ilike(search_term.clone()))
                    .or(message.ilike(search_term))
            );
        }
    }

    let results = messages_query
        .order(id.desc())
        .limit(per_page)
        .offset(offset)
        .load::<ContactMessage>(&mut conn)
        .await?;

    let b_count = blogs.count().get_result::<i64>(&mut conn).await.unwrap_or(0);
    let e_count = experiences
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .unwrap_or(0);
    let m_count = count_query.count().get_result::<i64>(&mut conn).await.unwrap_or(0);
    let s_count = social_links
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .unwrap_or(0);

    let total_pages = if m_count == 0 { 1 } else { (m_count + per_page - 1) / per_page };
    let pages_vec: Vec<i64> = (1..=total_pages).collect();

    let context = AdminHomeTemplate {
        messages: results,
        blog_count: b_count,
        experience_count: e_count,
        message_count: m_count,
        social_count: s_count,
        active_nav: "dashboard".to_string(),
        current_page: page,
        total_pages,
        pages: pages_vec,
        search_query: pagination.query,
    };

    Ok(Html(context.render()?))
}

pub async fn social_link_delete_handler(
    State(state): State<AppState>,
    Path(data_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    use crate::schema::social_links::dsl::*;

    diesel::delete(social_links.filter(id.eq(data_id)))
        .execute(&mut conn)
        .await?;

    Ok(Redirect::to("/auth/admin-panel"))
}

pub async fn message_delete_handler(
    State(state): State<AppState>,
    Path(message_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    use crate::schema::messages::dsl::*;

    diesel::delete(messages.filter(id.eq(message_id)))
        .execute(&mut conn)
        .await?;

    Ok(Redirect::to("/auth/admin-panel"))
}

#[derive(Template, Deserialize)]
#[template(path = "admin/add_social_link.html")]
struct SocialLinkCreateTemplate {
    social_links: Vec<SocialLink>,
    active_nav: String,
}

pub async fn social_link_create_page(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    use crate::schema::social_links::dsl::*;

    let links = social_links
        .load::<SocialLink>(&mut conn)
        .await?;

    let context = SocialLinkCreateTemplate {
        social_links: links,
        active_nav: "social".to_string(),
    };
    Ok(Html(context.render()?))
}

#[derive(Deserialize, Debug)]
pub struct SocialMediaForm {
    social_media: String,
    social_link: String,
}

pub async fn social_link_create_handler(
    State(state): State<AppState>,
    Form(form_data): Form<SocialMediaForm>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("{} - {}", form_data.social_media, form_data.social_link);
    let new_social_link = NewSocialLink {
        social_media: form_data.social_media.as_str(),
        social_link: form_data.social_link.as_str(),
    };
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    use crate::schema::social_links::dsl::*;
    diesel::insert_into(social_links)
        .values(&new_social_link)
        .execute(&mut conn)
        .await?;
    Ok(Redirect::to("/auth/admin-panel"))
}

#[derive(Template, Deserialize)]
#[template(path = "admin/update_social_link.html")]
struct SocialLinkUpdateTemplate {
    social_link: SocialLink,
    active_nav: String,
}

pub async fn social_link_update_page(
    State(state): State<AppState>,
    Path(data_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    use crate::schema::social_links::dsl::*;
    let this_social_link = social_links
        .filter(id.eq(data_id))
        .limit(1)
        .first::<SocialLink>(&mut conn)
        .await?;

    let context = SocialLinkUpdateTemplate {
        social_link: this_social_link,
        active_nav: "social".to_string(),
    };
    Ok(Html(context.render()?))
}

pub async fn social_link_update_handler(
    State(state): State<AppState>,
    Path(data_id): Path<i32>,
    Form(form_data): Form<SocialMediaForm>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("{} - {}", form_data.social_media, form_data.social_link);

    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    use crate::schema::social_links::dsl::*;

    let update_social_link = UpdateSocialLink {
        social_media: Some(form_data.social_media),
        social_link: Some(form_data.social_link),
    };

    diesel::update(social_links.filter(id.eq(data_id)))
        .set(update_social_link)
        .execute(&mut conn)
        .await?;
    Ok(Redirect::to("/auth/admin-panel"))
}


pub async fn logout_handler() -> Result<impl IntoResponse, AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str("session_id=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0").map_err(|e| AppError::Internal(e.to_string()))?,
    );

    tracing::info!("User logged out successfully...");
    Ok((headers, Redirect::to("/auth/login")))
}
