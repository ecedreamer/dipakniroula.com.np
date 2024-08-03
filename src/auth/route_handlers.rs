use std::path::Path;
use serde::Deserialize;

use askama::Template;
use axum::{http::StatusCode, response::{Html, IntoResponse, Redirect}, routing::{get, post}, Form, Router};
use diesel::RunQueryDsl;
use tokio::fs;
use tokio::fs::File;
use tower_sessions::Session;
use tokio::io::AsyncReadExt;
use crate::auth::models::NewSocialLink;
use crate::db::establish_connection;
use crate::schema::social_links;

pub async fn auth_routes() -> Router {
    Router::new()
    .route("/login", get(login_page))
    .route("/login", post(login_handler))
    .route("/admin-panel", get(admin_home_page))
    .route("/add-social-link", get(social_link_create_page))
    .route("/add-social-link", post(social_link_create_handler))
}


#[derive(Template, Deserialize)]
#[template(path = "login.html")]
struct LoginTemplate {
    page: String
}

pub async fn login_page() -> impl IntoResponse {
    let context = LoginTemplate {
        page: "Login".to_owned()
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
struct LoginForm {
    email: String,
    password: String,
}


pub async fn login_handler(session: Session, Form(form_data): Form<LoginForm>) -> impl IntoResponse {

    tracing::info!("Hello, {}! Your email is {}", form_data.email, form_data.password);
    if form_data.email == "sangit.niroula@gmail.com".to_string() && form_data.password == "admin@123".to_string() {
        session.insert("username", "sangit.niroula@gmail.com").await.unwrap();
        Redirect::to("/auth/admin-panel")
    } else{
        Redirect::to("/auth/login")
    }
}


#[derive(Template, Deserialize)]
#[template(path = "admin_home.html")]
struct AdminHomeTemplate {
    page: String,
    username: String
}

pub async fn admin_home_page(session: Session) -> impl IntoResponse {

    let username: Option<String> = session.get("username").await.unwrap();

    match username {
        Some(uname) => {
            let context = AdminHomeTemplate {
                page: "Admin Home".to_owned(),
                username: uname.to_string()
            };


            match context.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to render HTML".to_string(),
                )
                    .into_response()
            }
        },
        None => {
            Redirect::to("/auth/login").into_response()
        }
    }
}


#[derive(Template, Deserialize)]
#[template(path = "add_social_link.html")]
struct SocialLinkCreateTemplate {
    page: String,
}

pub async fn social_link_create_page() -> impl IntoResponse {
    let context = SocialLinkCreateTemplate {
        page: "Social Link Create".to_string()
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
    social_link: String
}


pub async fn social_link_create_handler(Form(form_data): Form<SocialMediaForm>) -> impl IntoResponse {
    tracing::info!("{} - {}", form_data.social_media, form_data.social_link);
    let social_link = NewSocialLink {
        social_media: form_data.social_media.as_str(),
        social_link: form_data.social_link.as_str(),
    };
    let conn = &mut establish_connection().await;
    diesel::insert_into(social_links::table)
        .values(&social_link)
        .execute(conn).unwrap();
    Redirect::to("/auth/admin-panel").into_response()
}