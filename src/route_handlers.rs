use askama::Template;
use diesel_async::RunQueryDsl;

use axum::{
    extract::Multipart,
    http::StatusCode,
    Form,
    Json,
    response::{Html, IntoResponse, Redirect},
};
use tokio::{
    fs::File,
    io::AsyncWriteExt,
};


use chrono::Utc;
use axum_csrf::CsrfToken;
use serde::Deserialize;
use serde_json::json;


use crate::blog::blog_repository::blog_repo::BlogRepository;
use crate::blog::models::Blog;
use crate::db::establish_connection;


#[derive(Template, Deserialize)]
#[template(path = "home.html")]
struct HomeTemplate {
    popular_blogs: Vec<Blog>,
}

pub async fn home_page() -> impl IntoResponse {
    let connection = &mut establish_connection().await;

    let blog_repo = BlogRepository::new(connection);

    let results = blog_repo.find_active_only(None, "view_count", 3).await;


    let context = HomeTemplate {
        popular_blogs: results.unwrap(),
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
    authenticity_token: String,
}


pub async fn contact_page(token: CsrfToken) -> impl IntoResponse {
    let context = ContactTemplate {
        authenticity_token: token.authenticity_token().unwrap(),
    };
    match context.render() {
        Ok(html) => (token, Html(html)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        ).into_response(),
    }
}


#[derive(Deserialize, Debug)]
pub struct ContactForm {
    authenticity_token: String,
    full_name: String,
    email: String,
    mobile: Option<String>,
    subject: String,
    message: String,
    website_url: Option<String>,
}

pub async fn contact_form_handler(token: CsrfToken, Form(form): Form<ContactForm>) -> impl IntoResponse {
    tracing::info!("Contact form submission from: {}", form.full_name);
    
    // Honeypot check
    if form.website_url.is_some() && !form.website_url.as_ref().unwrap().is_empty() {
        tracing::warn!("Honeypot triggered by: {}", form.full_name);
        return Redirect::to("/contact").into_response();
    }

    if token.verify(&form.authenticity_token).is_err() {
        tracing::error!("Invalid CSRF token from: {}", form.full_name);
        return Redirect::to("/contact").into_response();
    }

    // Basic length validation
    if form.full_name.len() > 100 || form.email.len() > 100 || form.subject.len() > 200 || form.message.len() > 5000 {
        tracing::error!("Input too long from: {}", form.full_name);
        return (StatusCode::BAD_REQUEST, "Input too long").into_response();
    }

    let date_time_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mobile = form.mobile.unwrap_or_default();

    if false { // Placeholder logic to keep structure if needed, but normally verified above
        Redirect::to("/contact").into_response()
    } else {
        use crate::schema::messages;

        let contact_message = crate::models::NewContactMessage {
            full_name: form.full_name.as_str(),
            email: form.email.as_str(),
            mobile: Some(mobile.as_str()),
            subject: form.subject.as_str(),
            message: form.message.as_str(),
            date_sent: date_time_str.as_str(),
        };
        let conn = &mut establish_connection().await;

        diesel::insert_into(messages::table)
            .values(&contact_message)
            .execute(conn)
            .await
            .unwrap();

        Redirect::to("/").into_response()
    }
}


pub async fn summernote_upload(mut multipart: Multipart) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut image_path = String::new();
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap().to_string();
        tracing::info!("{}", field_name);

        if field_name == "file" {
            let file_name = field.file_name().unwrap().to_string();

            if !file_name.is_empty() {
                let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
                image_path = format!("{}{}_{}", "media/summernote/", timestamp, file_name);
                let mut file = File::create(image_path.clone()).await.unwrap();
                while let Some(chunk) = field.chunk().await.unwrap() {
                    file.write_all(&chunk).await.unwrap();
                }
            }
        }
    }
    Ok(Json(json!({"image_path": "/".to_owned() + &*image_path})))
}