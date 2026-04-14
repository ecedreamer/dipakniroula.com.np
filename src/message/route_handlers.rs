use std::str::FromStr;
use askama::Template;
use axum::extract::{Path, Query};
use axum::response::{Html, IntoResponse, Redirect};
use axum::{Form, Router};
use axum::routing::get;
use axum::http::StatusCode;
use serde::Deserialize;
use crate::db::establish_connection;
use crate::middlewares::session_middleware;
use crate::message::message_repository::MessageRepository;
use crate::message::models::Message;

pub async fn message_routes() -> Router<axum_csrf::CsrfConfig> {
    Router::new()
        .route("/admin/messages", get(admin_message_list_page).layer(axum::middleware
