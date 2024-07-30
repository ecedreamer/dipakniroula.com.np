use std::collections::HashMap;
use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use serde::Deserialize;
use serde_json::{from_str, Value};
use tokio::fs::read_to_string;

#[derive(Template, Deserialize)]
#[template(path = "home.html")]
struct HomeTemplate {
    name: String,
    current_company: String,
    current_position: String,
    company_link: String,
    skills: HashMap<String, Vec<String>>,
}

pub async fn home_page() -> impl IntoResponse {
    let json_str = read_to_string("profile.json").await.unwrap();
    let content: Value = from_str(&json_str).unwrap();

    let name = content["general_introduction"]["name"].as_str().unwrap().to_string();
    let current_company = content["general_introduction"]["current_company"].as_str().unwrap().to_string();
    let current_position = content["general_introduction"]["current_position"].as_str().unwrap().to_string();
    let company_link = content["general_introduction"]["company_link"].as_str().unwrap().to_string();
    let skills= content["skills"].as_object().unwrap().iter()
        .map(|(k, v)| {
            (k.clone(), v.as_array().unwrap().iter().map(|s| s.as_str().unwrap().to_string()).collect())
        })
        .collect();


    let context = HomeTemplate {
        name,
        current_company,
        current_position,
        company_link,
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


pub async fn blog_list_page() -> impl IntoResponse {
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
    github: String,
    youtube: Option<String>,
    facebook: Option<String>,
}


pub async fn contact_page() -> impl IntoResponse {
    let context = ContactTemplate {
        mobile: "9849978896".to_string(),
        email: "sangit.niroula@gmail.com".to_string(),
        linkedin: "https://www.linkedin.com/in/dipak-niroula-90b11610b/".to_string(),
        github: "https://github.com/ecedreamer/".to_string(),
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

pub async fn render_template() -> impl IntoResponse {
    let template = HeaderTemplate.render().unwrap();
    Html(template)
}