use std::str::FromStr;
use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum::{Form, Router};
use axum::extract::Path;
use axum::routing::get;
use crate::db::establish_connection;
use crate::resume::models::{Experience, NewExperience, UpdateExperience};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use crate::auth::models::SocialLink;
use crate::middlewares::session_middleware;
use crate::resume::resume_repository::ExperienceRepository;

pub async fn resume_routes() -> Router {
    
    Router::new()
        // client side pages
        .route("/my-resume", get(resume_page))

        // admin side pages
        .route("/admin/experience/list",
               get(admin_experience_list_page).layer(axum::middleware::from_fn(session_middleware)))
        .route("/admin/experience/create",
               get(create_experience_page)
                   .post(handle_create_experience).layer(axum::middleware::from_fn(session_middleware)))
        .route("/admin/experience/{exp_id}/update",
               get(update_experience_page)
                   .post(handle_update_experience).layer(axum::middleware::from_fn(session_middleware)))
        .route("/admin/experience/{exp_id}/delete",
               get(handle_delete_experience).layer(axum::middleware::from_fn(session_middleware)))

}


#[derive(Template)]
#[template(path = "resume.html")]
pub struct ResumeTemplate {
    experiences: Vec<Experience>,
    social_links: Vec<SocialLink>,
}


pub async fn resume_page() -> impl IntoResponse {
    let conn = &mut establish_connection().await;


    use crate::schema::social_links::dsl::social_links;
    let my_social_links = social_links
        .select(SocialLink::as_select())
        .load(conn)
        .await
        .expect("Error loading social links");

    let repo = ExperienceRepository::new(conn);
    let result = repo.find().await;


    match result {
        Ok(experience_list) => {
            let context = ResumeTemplate {
                experiences: experience_list,
                social_links: my_social_links
            };

            match context.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to render HTML".to_string(),
                ).into_response(),
            }
        },
        Err(_) => {
            tracing::error!("Error fetching experiences");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to render HTML".to_string(),
            ).into_response()
        }
    }
}



#[derive(Template)]
#[template(path = "admin/experiencelist.html")]
pub struct AdminExperienceListTemplate {
    experience_list: Vec<Experience>,
    active_nav: String,
}


pub async fn admin_experience_list_page() -> impl IntoResponse {
    let conn = &mut establish_connection().await;
    let repo = ExperienceRepository::new(conn);


    let result = repo.find().await;

    match result {
        Ok(experience_list) => {
            let context = AdminExperienceListTemplate {
                experience_list,
                active_nav: "experiences".to_string(),
            };
            match context.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to render HTML".to_string(),
                ).into_response(),
            }
        },
        Err(err) => {
            tracing::error!("Could not fetch experience list; error: {:?}", err);
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
    active_nav: String,
}

pub async fn create_experience_page() -> impl IntoResponse {
    let context = CreateExperienceTemplate {
        active_nav: "experiences".to_string(),
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
        order: form_data.order

    };
    let conn = &mut establish_connection().await;

    let repo = ExperienceRepository::new(conn);
    match repo.insert_one(&experience).await {
        Ok(_) => Redirect::to("/auth/admin-panel").into_response(),
        Err(e) => {
            tracing::error!("Could not save the experience; error: {:?}", e);
            Redirect::to("/auth/admin-panel").into_response()
        }
    }
}


#[derive(Template)]
#[template(path = "admin/updateexperience.html")]
pub struct UpdateExperienceTemplate {
    experience: Experience,
    active_nav: String,
}

pub async fn update_experience_page(Path(exp_id): Path<String>) -> impl IntoResponse {
    let conn = &mut establish_connection().await;
    let repo = ExperienceRepository::new(conn);

    let exp_id_num = match i32::from_str(&exp_id) {
        Ok(num) => num,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid experience ID").into_response(),
    };

    match repo.find_by_id(exp_id_num).await {
        Ok(experience) => {
            let context = UpdateExperienceTemplate { 
                experience,
                active_nav: "experiences".to_string(),
            };
            match context.render() {
                Ok(html) => Html(html).into_response(),
                Err(e) => {
                    tracing::error!("Template render error: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to render template").into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!("Database fetch error for ID {}: {:?}", exp_id_num, e);
            (StatusCode::NOT_FOUND, "Experience not found").into_response()
        }
    }
}


pub async fn handle_update_experience(Path(data_id): Path<String>, Form(form_data): Form<NewExperience>) -> impl IntoResponse {
    let conn = &mut establish_connection().await;
    let repo = ExperienceRepository::new(conn);

    let data_id_num = match i32::from_str(&data_id) {
        Ok(num) => num,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid experience ID").into_response(),
    };

    // Correctly map empty form strings to None, and non-empty to Some
    let experience = UpdateExperience {
        company_name: Some(form_data.company_name),
        company_link: Some(form_data.company_link),
        your_position: Some(form_data.your_position),
        start_date: Some(form_data.start_date),
        end_date: form_data.end_date.filter(|s| !s.trim().is_empty()),
        responsibility: form_data.responsibility.filter(|s| !s.trim().is_empty()),
        skills: form_data.skills.filter(|s| !s.trim().is_empty()),
        order: Some(form_data.order),
    };

    match repo.update_one(data_id_num, &experience).await {
        Ok(_) => Redirect::to("/admin/experience/list").into_response(),
        Err(e) => {
            tracing::error!("Update failed for ID {}: {:?}", data_id_num, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Update failed").into_response()
        }
    }


}


pub async fn handle_delete_experience(Path(exp_id): Path<String>) -> impl IntoResponse {
    let conn = &mut establish_connection().await;
    let repo = ExperienceRepository::new(conn);

    let exp_id_num = match i32::from_str(&exp_id) {
        Ok(num) => num,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid experience ID").into_response(),
    };

    match repo.delete_one(exp_id_num).await {
        Ok(_) => Redirect::to("/admin/experience/list").into_response(),
        Err(e) => {
            tracing::error!("Delete failed for ID {}: {:?}", exp_id_num, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Delete failed").into_response()
        }
    }
}