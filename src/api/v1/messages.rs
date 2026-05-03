use axum::{extract::State, Json, http::StatusCode};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{state::AppState, schema::messages};
use crate::models::ContactMessage;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct MessageDto {
    pub id: i32,
    pub full_name: String,
    pub email: String,
    pub mobile: Option<String>,
    pub subject: String,
    pub message: String,
    pub date_sent: String,
}

impl From<ContactMessage> for MessageDto {
    fn from(msg: ContactMessage) -> Self {
        MessageDto {
            id: msg.id.unwrap_or(0),
            full_name: msg.full_name,
            email: msg.email,
            mobile: msg.mobile,
            subject: msg.subject,
            message: msg.message,
            date_sent: msg.date_sent,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/messages",
    responses(
        (status = 200, description = "List of all messages", body = [MessageDto]),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("ApiKeyAuth" = [])
    )
)]
pub async fn get_messages(State(state): State<AppState>) -> Result<Json<Vec<MessageDto>>, StatusCode> {
    use crate::schema::messages::dsl::*;

    let mut conn = state.get_conn().await.map_err(|e| {
        tracing::error!("DB Connection error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let results = messages
        .order(id.desc())
        .load::<ContactMessage>(&mut conn)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch messages: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(results.into_iter().map(MessageDto::from).collect()))
}
