use argon2::{Argon2, PasswordHash, PasswordVerifier};
use serde::Deserialize;
use std::str::FromStr;

use diesel::prelude::*;

use crate::auth::models::{NewSocialLink, SocialLink, UpdateSocialLink};
use crate::db::establish_connection;
use crate::middlewares::session_middleware;
use crate::models::{AdminUser, ContactMessage, CustomSession};
use crate::session_backend::create_session;
use askama::Template;
use axum::{
    Extension, Form, Router,
    extract::Path,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use axum_csrf::CsrfToken;
use diesel_async::RunQueryDsl;

pub async fn auth_routes() -> Router<axum_csrf::CsrfConfig> {
    Router::new()
        .route("/login", get(login_page))
        .route("/login", post(login_handler))
        .route(
            "/admin-panel",
            get(admin_home_page).layer(axum::middleware::from_fn(session_middleware)),
        )
        .route(
            "/add-social-link",
            get(social_link_create_page)
                .post(social_link_create_handler)
                .layer(axum::middleware::from_fn(session_middleware)),
        )
        .route(
            "/update-social-link/{data_id}",
            get(social_link_update_page)
                .post(social_link_update_handler)
                .layer(axum::middleware::from_fn(session_middleware)),
        )
        .route(
            "/delete-social-link/{data_id}",
            get(social_link_delete_handler).layer(axum::middleware::from_fn(session_middleware)),
        )
        .route(
            "/delete-message/{message_id}",
            get(message_delete_handler).layer(axum::middleware::from_fn(session_middleware)),
        )
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    pub authenticity_token: String,
}

pub async fn login_page(token: CsrfToken) -> impl IntoResponse {
    let context = LoginTemplate {
        authenticity_token: token.authenticity_token().unwrap(),
    };

    match context.render() {
        Ok(html) => (token, Html(html)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        )
            .into_response(),
    }
}

#[derive(Deserialize, Debug)]
pub struct LoginForm {
    authenticity_token: String,
    email: String,
    password: String,
}

pub async fn login_handler(
    token: CsrfToken,
    Form(form_data): Form<LoginForm>,
) -> impl IntoResponse {
    if token.verify(&form_data.authenticity_token).is_err() {
        tracing::error!("Suspicious Payload: {}", &form_data.authenticity_token);
        return Redirect::to("/auth/login").into_response();
    }

    let mut conn = establish_connection().await;

    use crate::schema::admin_users::dsl::*;

    let result = admin_users
        .filter(email.eq(&form_data.email))
        .limit(1)
        .first::<AdminUser>(&mut conn)
        .await;

    match result {
        Ok(admin_user) => {
            let parsed_hash = PasswordHash::new(&admin_user.password).unwrap();
            match Argon2::default().verify_password(form_data.password.as_bytes(), &parsed_hash) {
                Ok(_) => {
                    // session.insert("email", admin_user.email).await.unwrap();
                    let session_obj = create_session(&mut conn, admin_user.email)
                        .await
                        .expect("Failed to create session");

                    let mut headers = HeaderMap::new();
                    let cookie_value = format!(
                        "session_id={}; HttpOnly; SameSite=Lax; Path=/",
                        session_obj.session_id
                    );
                    headers.insert(
                        header::SET_COOKIE,
                        HeaderValue::from_str(&cookie_value).unwrap(),
                    );

                    tracing::info!("Successfully logged in...");
                    let redirect_response = Redirect::to("/auth/admin-panel");
                    (headers, redirect_response).into_response()
                }
                Err(_) => {
                    tracing::error!("Invalid credentials...");
                    Redirect::to("/auth/login").into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!("Invalid credentials; Error: {}", e);
            Redirect::to("/auth/login").into_response()
        }
    }
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
}

pub async fn admin_home_page(Extension(_session): Extension<CustomSession>) -> impl IntoResponse {
    let conn = &mut establish_connection().await;

    use crate::schema::messages::dsl::*;
    use crate::schema::blogs::dsl::blogs;
    use crate::schema::experiences::dsl::experiences;
    use crate::schema::social_links::dsl::social_links;

    let results = messages
        .order(id.desc())
        .load::<ContactMessage>(conn)
        .await
        .expect("Error loading messages");

    let b_count = blogs.count().get_result::<i64>(conn).await.unwrap_or(0);
    let e_count = experiences.count().get_result::<i64>(conn).await.unwrap_or(0);
    let m_count = messages.count().get_result::<i64>(conn).await.unwrap_or(0);
    let s_count = social_links.count().get_result::<i64>(conn).await.unwrap_or(0);

    let context = AdminHomeTemplate {
        messages: results,
        blog_count: b_count,
        experience_count: e_count,
        message_count: m_count,
        social_count: s_count,
        active_nav: "dashboard".to_string(),
    };

    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        )
            .into_response(),
    }
}

pub async fn social_link_delete_handler(
    Path(data_id): Path<String>,
) -> impl IntoResponse {
    let conn = &mut establish_connection().await;
    use crate::schema::social_links::dsl::*;
    let data_id_num = i32::from_str(data_id.as_str()).unwrap();

    diesel::delete(social_links.filter(id.eq(data_id_num)))
        .execute(conn)
        .await
        .unwrap();

    Redirect::to("/auth/admin-panel").into_response()
}

pub async fn message_delete_handler(
    Path(message_id): Path<String>,
) -> impl IntoResponse {
    let conn = &mut establish_connection().await;
    use crate::schema::messages::dsl::*;
    let message_id_num = i32::from_str(message_id.as_str()).unwrap();

    diesel::delete(messages.filter(id.eq(message_id_num)))
        .execute(conn)
        .await
        .unwrap();

    Redirect::to("/auth/admin-panel").into_response()
}

#[derive(Template, Deserialize)]
#[template(path = "admin/add_social_link.html")]
struct SocialLinkCreateTemplate {
    social_links: Vec<SocialLink>,
    active_nav: String,
}

pub async fn social_link_create_page() -> impl IntoResponse {
    let conn = &mut establish_connection().await;
    use crate::schema::social_links::dsl::*;
    
    let links = social_links
        .load::<SocialLink>(conn)
        .await
        .expect("Error loading social links");

    let context = SocialLinkCreateTemplate {
        social_links: links,
        active_nav: "social".to_string(),
    };
    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        )
            .into_response(),
    }
}

#[derive(Deserialize, Debug)]
pub struct SocialMediaForm {
    social_media: String,
    social_link: String,
}

pub async fn social_link_create_handler(
    Form(form_data): Form<SocialMediaForm>,
) -> impl IntoResponse {
    tracing::info!("{} - {}", form_data.social_media, form_data.social_link);
    let new_social_link = NewSocialLink {
        social_media: form_data.social_media.as_str(),
        social_link: form_data.social_link.as_str(),
    };
    let conn = &mut establish_connection().await;
    use crate::schema::social_links::dsl::*;
    diesel::insert_into(social_links)
        .values(&new_social_link)
        .execute(conn)
        .await
        .unwrap();
    Redirect::to("/auth/admin-panel").into_response()
}

#[derive(Template, Deserialize)]
#[template(path = "admin/update_social_link.html")]
struct SocialLinkUpdateTemplate {
    social_link: SocialLink,
    active_nav: String,
}

pub async fn social_link_update_page(
    Path(data_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let conn = &mut establish_connection().await;
    use crate::schema::social_links::dsl::*;
    let data_id_num = i32::from_str(data_id.as_str()).unwrap();
    let this_social_link = social_links
        .filter(id.eq(data_id_num))
        .limit(1)
        .first::<SocialLink>(conn)
        .await
        .expect("Could not find social link");

    let context = SocialLinkUpdateTemplate {
        social_link: this_social_link,
        active_nav: "social".to_string(),
    };
    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        )
            .into_response(),
    }
}

pub async fn social_link_update_handler(
    Path(data_id): Path<String>,
    Form(form_data): Form<SocialMediaForm>,
) -> impl IntoResponse {
    tracing::info!("{} - {}", form_data.social_media, form_data.social_link);

    let conn = &mut establish_connection().await;
    use crate::schema::social_links::dsl::*;

    let update_social_link = UpdateSocialLink {
        social_media: Some(form_data.social_media),
        social_link: Some(form_data.social_link),
    };

    let data_id_num = i32::from_str(data_id.as_str()).unwrap();

    let target = social_links.filter(id.eq(data_id_num));

    diesel::update(target)
        .set(update_social_link)
        .execute(conn)
        .await
        .unwrap();
    Redirect::to("/auth/admin-panel").into_response()
}
