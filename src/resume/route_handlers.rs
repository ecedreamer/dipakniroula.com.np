use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::Router;
use axum::routing::get;
use crate::db::establish_connection;
use crate::resume::models::Experience;

use diesel::prelude::*;
use diesel::RunQueryDsl;
use crate::models::SocialLink;
use crate::resume::resume_repository::ExperienceRepository;

pub async fn resume_routes() -> Router {
    Router::new()
        // client side pages
        .route("/my-resume", get(resume_page))

}



#[derive(Template)]
#[template(path = "resume.html")]
pub struct ResumeTemplate {
    page: String,
    experiences: Vec<Experience>
}


pub async fn resume_page() -> impl IntoResponse {
    let conn = &mut establish_connection().await;
    let repo = ExperienceRepository::new(conn);


    let result = repo.find();

    match result {
        Ok(experience_list) => {
            let context = ResumeTemplate {
                page: "My Resume".to_string(),
                experiences: experience_list
            };

            match context.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to render HTML".to_string(),
                ).into_response(),
            }
        },
        Err(e) => {
            tracing::error!("Error fetching experiences");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to render HTML".to_string(),
            ).into_response()
        }
    }


}