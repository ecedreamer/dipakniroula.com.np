use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(diesel::result::Error),
    TemplateError(askama::Error),
    IoError(std::io::Error),
    NotFound(String),
    Internal(String),
}

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<askama::Error> for AppError {
    fn from(err: askama::Error) -> Self {
        AppError::TemplateError(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(err) => {
                error!("Database error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::TemplateError(err) => {
                error!("Template rendering error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to render page".to_string())
            }
            AppError::IoError(err) => {
                error!("IO error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "File system error".to_string())
            }
            AppError::NotFound(item) => {
                (StatusCode::NOT_FOUND, format!("{} not found", item))
            }
            AppError::Internal(msg) => {
                error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };

        (status, error_message).into_response()
    }
}
