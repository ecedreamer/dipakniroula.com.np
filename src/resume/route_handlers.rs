use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::Router;
use axum::routing::get;



pub async fn resume_routes() -> Router {
    Router::new()
        // client side pages
        .route("/my-resume", get(resume_page))

}



#[derive(Template)]
#[template(path = "resume.html")]
pub struct ResumeTemplate {
    page: String,
}


pub async fn resume_page() -> impl IntoResponse {
    let context = ResumeTemplate {
        page: "My Resume".to_string(),
    };

    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        ).into_response(),
    }
}