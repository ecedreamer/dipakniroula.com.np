use diesel::prelude::*;


use serde::Deserialize;


use askama::Template;
use axum::{http::StatusCode, response::{Html, IntoResponse, Redirect}, routing::{get, post}, Form, Router};
use axum::extract::Multipart;
use crate::db::establish_connection;
use crate::models::{Blogs, NewBlog};
use diesel::RunQueryDsl;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::filter::{truncate_words, strip_tags};
use crate::middlewares::auth_middleware;

pub async fn blog_routes() -> Router {
    Router::new()
        .route("/list", get(blog_list_page))
        .route("/create", get(blog_create_page).layer(axum::middleware::from_fn(auth_middleware)))
        .route("/create", post(blog_create_handler).layer(axum::middleware::from_fn(auth_middleware)))
}


#[derive(Template, Deserialize)]
#[template(path = "blogcreate.html")]
struct BlogCreateTemplate {
    page: String,
}

pub async fn blog_create_page() -> impl IntoResponse {
    let context = BlogCreateTemplate {
        page: "Blog Create".to_owned()
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


#[derive(Debug, Deserialize)]
pub struct BlogForm {
    title: String,
    content: String,
}


pub async fn blog_create_handler(mut multipart: Multipart) -> impl IntoResponse {
    tracing::info!("Handling multipart request");
    let mut image_path = String::new();
    let mut title = String::new();
    let mut content = String::new();
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap().to_string();
        tracing::info!("{}", field_name);

        if field_name == "blog-image" {
            let file_name = field.file_name().unwrap().to_string();
            image_path = format!("{}{}", "media/", file_name);

            // Save the file
            let mut file = File::create(image_path.clone()).await.unwrap();
            while let Some(chunk) = field.chunk().await.unwrap() {
                file.write_all(&chunk).await.unwrap();
            }
        } else if field_name == "title" {
            title = field.text().await.unwrap();
        } else if field_name == "content" {
            content = field.text().await.unwrap();
        }
    }

    use crate::schema::blogs::dsl::blogs;

    let conn = &mut establish_connection().await;

    let blog = NewBlog {
        title: &title,
        content: &content,
        image: Some(&image_path)
    };

    diesel::insert_into(blogs)
        .values(&blog)
        .execute(conn)
        .unwrap();

    Redirect::to("/auth/admin-panel").into_response()
}

// pub async fn blog_create_handler(Form(form_data): Form<BlogForm>) -> impl IntoResponse {
//     tracing::info!("{:?}", form_data.title);
//     use crate::schema::blogs::dsl::blogs;
//
//     let conn= &mut establish_connection().await;
//
//     let blog = NewBlog{
//         title: form_data.title.as_str(),
//         content: form_data.content.as_str()
//     };
//
//     diesel::insert_into(blogs)
//         .values(&blog)
//         .execute(conn)
//         .unwrap();
//
//     Redirect::to("/auth/admin-panel").into_response()
// }


struct Blog {
    title: String,
    content: String,
}


#[derive(Template)]
#[template(path = "blog_list.html")]
struct BlogListTemplate {
    blog_list: Vec<Blogs>,
}


pub async fn blog_list_page() -> impl IntoResponse {
    use crate::schema::blogs::dsl::*;

    let mut conn = establish_connection().await;

    let results = blogs
        .load::<Blogs>(&mut conn)
        .expect("Error loading blogs");

    let context = BlogListTemplate {
        blog_list: results.to_vec(),
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

