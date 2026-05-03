use axum::{Router, routing::get, middleware};
use crate::state::AppState;
use crate::api::auth::require_api_key;

pub mod blogs;
pub mod experiences;
pub mod messages;

pub fn v1_routes() -> Router<AppState> {
    Router::new()
        .route("/blogs", get(blogs::get_blogs))
        .route("/blogs/{id}", get(blogs::get_blog_by_id))
        .route("/experiences", get(experiences::get_experiences))
        .route("/messages", get(messages::get_messages))
        .route_layer(middleware::from_fn(require_api_key))
}
