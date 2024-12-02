use askama::Template;
use axum::extract::Multipart;
use axum::{Form, Json};
use diesel::prelude::*;
use diesel::RunQueryDsl;
use chrono::Utc;


use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum_csrf::CsrfToken;
use serde::Deserialize;
use serde_json::json;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::auth::models::SocialLink;
use crate::blog::blog_repository::blog_repository::BlogRepository;
use crate::blog::models::Blog;
use crate::db::establish_connection;



#[derive(Template, Deserialize)]
#[template(path = "home.html")]
struct HomeTemplate {
    social_links: Vec<SocialLink>,
    popular_blogs: Vec<Blog>
}

pub async fn home_page() -> impl IntoResponse {

    let connection = &mut establish_connection().await;

    let blog_repo = BlogRepository::new(connection);

    let results = blog_repo.find_active_only(None, "view_count", 3);


    use crate::schema::social_links::dsl::social_links;
    let my_social_links = social_links
        .select(SocialLink::as_select())
        .load(connection)
        .expect("Error loading social links");


    let context = HomeTemplate {
        social_links: my_social_links,
        popular_blogs: results.unwrap()
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
    social_links: Vec<SocialLink>,
    authenticity_token: String,
}


pub async fn contact_page(token: CsrfToken) -> impl IntoResponse {
    let connection = &mut establish_connection().await;
    use crate::schema::social_links::dsl::social_links;
    let results = social_links
        .select(SocialLink::as_select())
        .load(connection)
        .expect("Error loading social links");


    let context = ContactTemplate {
        social_links: results,
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
}

pub async fn contact_form_handler(token: CsrfToken, Form(form): Form<ContactForm>) -> impl IntoResponse {
    tracing::info!("{} ------", form.full_name);
    let date_time_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mobile = form.mobile.unwrap_or(String::new());

    if token.verify(&form.authenticity_token).is_err() {
        tracing::error!("Suspicious Payload: {}", &form.authenticity_token);
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