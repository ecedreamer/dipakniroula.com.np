use axum::{extract::State, Json, http::StatusCode};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{state::AppState, schema::experiences};
use crate::resume::models::Experience;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ExperienceDto {
    pub id: i32,
    pub company_name: String,
    pub your_position: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub responsibility: Option<String>,
    pub skills: Option<String>,
    pub company_link: String,
    pub order: i32,
    pub duration_display: String,
}

impl From<Experience> for ExperienceDto {
    fn from(exp: Experience) -> Self {
        let duration = exp.duration();
        ExperienceDto {
            id: exp.id.unwrap_or(0),
            company_name: exp.company_name,
            your_position: exp.your_position,
            start_date: exp.start_date,
            end_date: exp.end_date,
            responsibility: exp.responsibility,
            skills: exp.skills,
            company_link: exp.company_link,
            order: exp.order,
            duration_display: duration,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/experiences",
    responses(
        (status = 200, description = "List of all experiences", body = [ExperienceDto]),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("ApiKeyAuth" = [])
    )
)]
pub async fn get_experiences(State(state): State<AppState>) -> Result<Json<Vec<ExperienceDto>>, StatusCode> {
    use crate::schema::experiences::dsl::*;

    let mut conn = state.get_conn().await.map_err(|e| {
        tracing::error!("DB Connection error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let results = experiences
        .order(order.asc())
        .load::<Experience>(&mut conn)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch experiences: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(results.into_iter().map(ExperienceDto::from).collect()))
}
