use askama::Template;
use axum::response::{Html, IntoResponse, Redirect};
use axum::{Form, Router, Extension};
use axum::extract::{Path, State};
use axum::routing::get;
use crate::resume::models::{Experience, NewExperience, UpdateExperience};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use crate::auth::models::SocialLink;
use crate::middlewares::session_middleware;
use crate::resume::resume_repository::ExperienceRepository;
use crate::state::AppState;
use crate::utils::error::AppError;

pub async fn resume_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // client side pages
        .route("/my-resume", get(resume_page))

        // admin side pages
        .route("/admin/experience/list",
               get(admin_experience_list_page).layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)))
        .route("/admin/experience/create",
               get(create_experience_page)
                   .post(handle_create_experience).layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)))
        .route("/admin/experience/{exp_id}/update",
               get(update_experience_page)
                   .post(handle_update_experience).layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)))
        .route("/admin/experience/{exp_id}/delete",
               get(handle_delete_experience).layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)))
}




#[derive(Template)]
#[template(path = "resume.html")]
pub struct ResumeTemplate {
    experiences: Vec<Experience>,
    social_links: Vec<SocialLink>,
    flash: Option<crate::models::FlashData>,
}


pub async fn resume_page(
    State(state): State<AppState>,
    session: Option<axum::Extension<crate::models::CustomSession>>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;

    let flash = if let Some(axum::Extension(s)) = session {
        crate::session_backend::take_flash(&mut conn, &s).await.1
    } else {
        crate::models::FlashData::default()
    };


    use crate::schema::social_links::dsl::social_links;
    let my_social_links = social_links
        .select(SocialLink::as_select())
        .load(&mut conn)
        .await?;

    let repo = ExperienceRepository::new(&mut conn);
    let results = repo.find().await?;


    let context = ResumeTemplate {
        experiences: results,
        social_links: my_social_links,
        flash: Some(flash),
    };

    Ok(Html(context.render()?))
}



#[derive(Template)]
#[template(path = "admin/experiencelist.html")]
pub struct AdminExperienceListTemplate {
    experience_list: Vec<Experience>,
    active_nav: String,
    flash: Option<crate::models::FlashData>,
}


pub async fn admin_experience_list_page(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<crate::models::CustomSession>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    
    let flash = crate::session_backend::take_flash(&mut conn, &session).await.1;
    let repo = ExperienceRepository::new(&mut conn);

    let results = repo.find().await?;

    let context = AdminExperienceListTemplate {
        experience_list: results,
        active_nav: "experiences".to_string(),
        flash: Some(flash),
    };
    
    Ok(Html(context.render()?))
}



#[derive(Template)]
#[template(path = "admin/createexperience.html")]
pub struct CreateExperienceTemplate {
    active_nav: String,
    flash: Option<crate::models::FlashData>,
}

pub async fn create_experience_page(
    State(state): State<AppState>,
    session: Option<Extension<crate::models::CustomSession>>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let flash = if let Some(Extension(s)) = session {
        crate::session_backend::take_flash(&mut conn, &s).await.1
    } else {
        crate::models::FlashData::default()
    };
    let context = CreateExperienceTemplate {
        active_nav: "experiences".to_string(),
        flash: Some(flash),
    };
    Ok(Html(context.render()?))
}


pub async fn handle_create_experience(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<crate::models::CustomSession>,
    Form(form_data): Form<NewExperience>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;

    let repo = ExperienceRepository::new(&mut conn);
    repo.insert_one(&form_data).await?;

    crate::session_backend::set_flash(&mut conn, &session, Some("Experience added successfully.".to_string()), None).await?;
    
    Ok(Redirect::to("/admin/experience/list"))
}


#[derive(Template)]
#[template(path = "admin/updateexperience.html")]
pub struct UpdateExperienceTemplate {
    experience: Experience,
    active_nav: String,
    flash: Option<crate::models::FlashData>,
}

pub async fn update_experience_page(
    State(state): State<AppState>,
    session: Option<Extension<crate::models::CustomSession>>,
    Path(exp_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    let flash = if let Some(Extension(s)) = session {
        crate::session_backend::take_flash(&mut conn, &s).await.1
    } else {
        crate::models::FlashData::default()
    };
    let repo = ExperienceRepository::new(&mut conn);

    let experience = repo.find_by_id(exp_id).await?;
    
    let context = UpdateExperienceTemplate { 
        experience,
        active_nav: "experiences".to_string(),
        flash: Some(flash),
    };
    Ok(Html(context.render()?))
}


pub async fn handle_update_experience(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<crate::models::CustomSession>,
    Path(data_id): Path<i32>, 
    Form(form_data): Form<NewExperience>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    let repo = ExperienceRepository::new(&mut conn);

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

    repo.update_one(data_id, &experience).await?;
    
    crate::session_backend::set_flash(&mut conn, &session, Some("Experience updated successfully.".to_string()), None).await?;

    Ok(Redirect::to("/admin/experience/list"))
}


pub async fn handle_delete_experience(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<crate::models::CustomSession>,
    Path(exp_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    let repo = ExperienceRepository::new(&mut conn);

    repo.delete_one(exp_id).await?;
    
    crate::session_backend::set_flash(&mut conn, &session, Some("Experience deleted successfully.".to_string()), None).await?;

    Ok(Redirect::to("/admin/experience/list"))
}