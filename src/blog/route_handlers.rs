use serde::Deserialize;

use askama::Template;
use axum::{http::StatusCode, response::{Html, IntoResponse, Redirect}, routing::{get, post}, Form, Router};
use crate::db::establish_connection;
use crate::models::NewBlog;
use diesel::RunQueryDsl;

pub async fn blog_routes() -> Router {
    Router::new()
        .route("/create", get(blog_create_page))
        .route("/create", post(blog_create_handler))
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