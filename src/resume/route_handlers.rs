use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum::{Form, Router};
use axum::routing::get;
use crate::db::establish_connection;
use crate::resume::models::{Experience, NewExperience};

use diesel::prelude::*;
use diesel::RunQueryDsl;
use crate::auth::route_handlers::SocialMediaForm;
use crate::models::SocialLink;
use crate::resume::models;
use crate::resume::resume_repository::ExperienceRepository;
use crate::schema::{experiences, social_links};

pub async fn resume_routes() -> Router {
    let routes = Router::new()
        // client side pages
        .route("/my-resume", get(resume_page))

        // admin side pages
        .route("/experience/admin/create",
               get(create_experience_page)
                   .post(handle_create_experience));
    routes

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


#[derive(Template)]
#[template(path = "admin/createexperience.html")]
pub struct CreateExperienceTemplate {
    page: String,
}

pub async fn create_experience_page() -> impl IntoResponse {
    let context = CreateExperienceTemplate {
        page: "Create Experience".to_string(),
    };
    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        ).into_response(),
    }
}


pub async fn handle_create_experience(Form(form_data): Form<NewExperience>) -> impl IntoResponse {
    let experience = NewExperience {
        company_name: form_data.company_name,
        company_link: form_data.company_link,
        your_position: form_data.your_position,
        start_date: form_data.start_date,
        end_date: form_data.end_date,
        responsibility: form_data.responsibility,
        skills: form_data.skills,

    };
    let conn = &mut establish_connection().await;

    diesel::insert_into(experiences::table)
        .values(&experience)
        .execute(conn).unwrap();
    Redirect::to("/auth/admin-panel").into_response()
}
