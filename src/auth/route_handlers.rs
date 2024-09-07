use std::str::FromStr;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use serde::Deserialize;

use diesel::prelude::*;

use askama::Template;
use axum::{http::StatusCode, response::{Html, IntoResponse, Redirect}, routing::{get, post}, Form, Router};
use axum::extract::Path;
use diesel::RunQueryDsl;
use tower_sessions::Session;
use crate::auth::models::{NewSocialLink, SocialLink, UpdateSocialLink};
use crate::db::establish_connection;
use crate::middlewares::auth_middleware;
use crate::models::{AdminUser, ContactMessage};


pub async fn auth_routes() -> Router {
    Router::new()
        .route("/login", get(login_page))
        .route("/login", post(login_handler))
        .route("/admin-panel", get(admin_home_page).layer(axum::middleware::from_fn(auth_middleware)))
        .route("/add-social-link",
               get(social_link_create_page)
                   .post(social_link_create_handler)
                   .layer(axum::middleware::from_fn(auth_middleware)))
        .route("/update-social-link/:data_id",
               get(social_link_update_page)
                   .post(social_link_update_handler)
                   .layer(axum::middleware::from_fn(auth_middleware)))
}


#[derive(Template, Deserialize)]
#[template(path = "login.html")]
struct LoginTemplate {
}

pub async fn login_page() -> impl IntoResponse {
    let context = LoginTemplate {
    };


    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        )
            .into_response()
    }
}


#[derive(Deserialize, Debug)]
pub struct LoginForm {
    email: String,
    password: String,
}


pub async fn login_handler(session: Session, Form(form_data): Form<LoginForm>) -> impl IntoResponse {
    let mut conn = establish_connection().await;


    use crate::schema::admin_users::dsl::*;

    let result = admin_users
        .filter(email.eq(&form_data.email))
        .limit(1)
        .first::<AdminUser>(&mut conn);

    match result {
        Ok(admin_user) => {
            let parsed_hash = PasswordHash::new(&admin_user.password).unwrap();
            match Argon2::default().verify_password(&form_data.password.as_bytes(), &parsed_hash) {
                Ok(_) => {
                    session.insert("email", admin_user.email).await.unwrap();
                    tracing::info!("Successfully logged in...");
                    Redirect::to("/auth/admin-panel")
                }
                Err(_) => {
                    tracing::error!("Invalid credentials...");
                    Redirect::to("/auth/login")
                }
            }
        }
        Err(e) => {
            tracing::error!("Invalid credentials; Error: {}", e);
            Redirect::to("/auth/login")
        }
    }
}


#[derive(Template, Deserialize)]
#[template(path = "admin/admin_home.html")]
struct AdminHomeTemplate {
    username: String,
    messages: Vec<ContactMessage>,
}

pub async fn admin_home_page(session: Session) -> impl IntoResponse {
    let user_email: Option<String> = session.get("email").await.unwrap();

    let conn = &mut establish_connection().await;

    use crate::schema::messages::dsl::*;

    let results = messages
        .order(id.desc())
        .load::<ContactMessage>(conn)
        .expect("Error loading blogs");

    match user_email {
        Some(u_email) => {
            let context = AdminHomeTemplate {
                username: u_email.to_string(),
                messages: results,
            };


            match context.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to render HTML".to_string(),
                )
                    .into_response()
            }
        }
        None => {
            Redirect::to("/auth/login").into_response()
        }
    }
}


#[derive(Template, Deserialize)]
#[template(path = "admin/add_social_link.html")]
struct SocialLinkCreateTemplate {
}

pub async fn social_link_create_page() -> impl IntoResponse {
    let context = SocialLinkCreateTemplate {
    };
    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR, "Failed to render HTML".to_string()
        ).into_response()
    }
}


#[derive(Deserialize, Debug)]
pub struct SocialMediaForm {
    social_media: String,
    social_link: String,
}


pub async fn social_link_create_handler(Form(form_data): Form<SocialMediaForm>) -> impl IntoResponse {
    tracing::info!("{} - {}", form_data.social_media, form_data.social_link);
    let new_social_link = NewSocialLink {
        social_media: form_data.social_media.as_str(),
        social_link: form_data.social_link.as_str(),
    };
    let conn = &mut establish_connection().await;
    use crate::schema::social_links::dsl::*;
    diesel::insert_into(social_links)
        .values(&new_social_link)
        .execute(conn).unwrap();
    Redirect::to("/auth/admin-panel").into_response()
}


#[derive(Template, Deserialize)]
#[template(path = "admin/update_social_link.html")]
struct SocialLinkUpdateTemplate {
    social_link: SocialLink
}

pub async fn social_link_update_page(Path(data_id): axum::extract::Path<String>) -> impl IntoResponse {
    let conn = &mut establish_connection().await;
    use crate::schema::social_links::dsl::*;
    let data_id_num = i32::from_str(data_id.as_str()).unwrap();
    let this_social_link = social_links
        .filter(id.eq(data_id_num))
        .limit(1)
        .first::<SocialLink>(conn)
        .expect("Could not find social link");

    let context = SocialLinkUpdateTemplate {
        social_link: this_social_link
    };
    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR, "Failed to render HTML".to_string()
        ).into_response()
    }
}



pub async fn social_link_update_handler(Path(data_id): Path<String>, Form(form_data): Form<SocialMediaForm>) -> impl IntoResponse {
    tracing::info!("{} - {}", form_data.social_media, form_data.social_link);

    let conn = &mut establish_connection().await;
    use crate::schema::social_links::dsl::*;

    let update_social_link = UpdateSocialLink {
        social_media: Some(form_data.social_media),
        social_link: Some(form_data.social_link)
    };

    let data_id_num = i32::from_str(data_id.as_str()).unwrap();

    let target = social_links.filter(id.eq(data_id_num));

    diesel::update(target)
        .set(update_social_link)
        .execute(conn)
        .unwrap();
    Redirect::to("/auth/admin-panel").into_response()
}