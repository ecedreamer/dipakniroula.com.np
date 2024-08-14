use diesel::prelude::*;


use serde::Deserialize;


use askama::Template;
use axum::{http::StatusCode, response::{Html, IntoResponse, Redirect}, routing::{get, post}, Form, Router};
use crate::db::establish_connection;
use crate::models::{Blogs, NewBlog};
use diesel::RunQueryDsl;
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
    page: String
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
struct BlogForm {
    title: String,
    content: String
}

pub async fn blog_create_handler(Form(form_data): Form<BlogForm>) -> impl IntoResponse {
    tracing::info!("{:?}", form_data.title);
    use crate::schema::blogs::dsl::blogs;

    let conn= &mut establish_connection().await;

    let blog = NewBlog{
        title: form_data.title.as_str(),
        content: form_data.content.as_str()
    };

    diesel::insert_into(blogs)
        .values(&blog)
        .execute(conn)
        .unwrap();

    Redirect::to("/auth/admin-panel").into_response()
}


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

