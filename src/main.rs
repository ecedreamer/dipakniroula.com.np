use axum::{routing::get, Router};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};

use askama::Template;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::error::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;


async fn read_file_to_string(path: &str) -> Result<Value, Box<dyn Error>> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let json: Value = serde_json::from_str(&contents)?;
    Ok(json)
}


#[derive(Template, Deserialize)]
#[template(path = "home.html")]
struct HomeTemplate {
    name: String,
    current_company: String,
    current_position: String,
}

async fn home_page() -> impl IntoResponse {
    let json_value = read_file_to_string("profile.json").await;
    let content = json_value.unwrap();
    let context: HomeTemplate = serde_json::from_value(content.get("general_introduction").unwrap().clone()).unwrap();

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
    blog_list: Vec<Blog>
}


async fn blog_list_page() -> impl IntoResponse {
    let blog1 = Blog {
        title: "First blog".to_string(),
        content: "This is the first blog".to_string()
    };
    let blog2 = Blog {
        title: "Second blog".to_string(),
        content: "This is the second blog".to_string()
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
        facebook: None
    };
    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        ).into_response(),
    }

}


#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(home_page))
        .route("/blogs/", get(blog_list_page))
        .route("/contact/", get(contact_page));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}