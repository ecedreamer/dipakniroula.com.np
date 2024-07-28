use std::collections::HashMap;
use axum::{routing::get, Router};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};

use askama::Template;
use tokio::fs::{File, read_to_string};
use tokio::io::AsyncReadExt;
use serde::Deserialize;
use serde_json::{from_str, Value};

use tower_http::services::ServeDir;


#[derive(Template, Deserialize)]
#[template(path = "home.html")]
struct HomeTemplate {
    name: String,
    current_company: String,
    current_position: String,
    skills: HashMap<String, Vec<String>>,
}

async fn home_page() -> impl IntoResponse {
    let json_str = read_to_string("profile.json").await.unwrap();
    let content: Value = from_str(&json_str).unwrap();

    let name = content["general_introduction"]["name"].as_str().unwrap().to_string();
    let current_company = content["general_introduction"]["current_company"].as_str().unwrap().to_string();
    let current_position = content["general_introduction"]["current_position"].as_str().unwrap().to_string();
    let skills= content["skills"].as_object().unwrap().iter()
        .map(|(k, v)| {
            (k.clone(), v.as_array().unwrap().iter().map(|s| s.as_str().unwrap().to_string()).collect())
        })
        .collect();


    let context = HomeTemplate {
        name,
        current_company,
        current_position,
        skills,
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


struct Blog {
    title: String,
    content: String,
}


#[derive(Template)]
#[template(path = "blog_list.html")]
struct BlogListTemplate {
    blog_list: Vec<Blog>,
}


async fn blog_list_page() -> impl IntoResponse {
    let blog1 = Blog {
        title: "First blog".to_string(),
        content: "This is the first blog".to_string(),
    };
    let blog2 = Blog {
        title: "Second blog".to_string(),
        content: "This is the second blog".to_string(),
    };
    let context = BlogListTemplate {
        blog_list: vec![blog1, blog2],
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


#[derive(Template)]
#[template(path = "contact.html")]
struct ContactTemplate {
    mobile: String,
    email: String,
    linkedin: String,
    youtube: Option<String>,
    facebook: Option<String>,
}


async fn contact_page() -> impl IntoResponse {
    let context = ContactTemplate {
        mobile: "9849978896".to_string(),
        email: "sangit.niroula@gmail.com".to_string(),
        linkedin: "https://www.linkedin.com/in/dipak-niroula-90b11610b/".to_string(),
        youtube: None,
        facebook: None,
    };
    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        ).into_response(),
    }
}

#[derive(Template)]
#[template(path = "header.html")]
struct HeaderTemplate;

async fn render_template() -> impl IntoResponse {
    let template = HeaderTemplate.render().unwrap();
    Html(template)
}


#[tokio::main]
async fn main() {
    let static_files_service = ServeDir::new("static");
    let app = Router::new()
        .route("/", get(home_page))
        .route("/blogs/", get(blog_list_page))
        .route("/contact/", get(contact_page))
        .nest_service("/static", static_files_service);
    // .fallback_service(ServiceBuilder::new().service(static_files_service));;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

