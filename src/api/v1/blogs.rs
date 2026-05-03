use axum::{extract::{State, Path}, Json, http::StatusCode};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{state::AppState, schema::blogs};
use crate::blog::models::Blog;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct BlogDto {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub image: Option<String>,
    pub published_date: String,
    pub modified_date: Option<String>,
    pub view_count: i32,
    pub is_active: i32,
}

impl From<Blog> for BlogDto {
    fn from(blog: Blog) -> Self {
        BlogDto {
            id: blog.id.unwrap_or(0),
            title: blog.title,
            content: blog.content,
            image: blog.image,
            published_date: blog.published_date,
            modified_date: blog.modified_date,
            view_count: blog.view_count,
            is_active: blog.is_active,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/blogs",
    responses(
        (status = 200, description = "List of all active blogs", body = [BlogDto]),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("ApiKeyAuth" = [])
    )
)]
pub async fn get_blogs(State(state): State<AppState>) -> Result<Json<Vec<BlogDto>>, StatusCode> {
    use crate::schema::blogs::dsl::*;

    let mut conn = state.get_conn().await.map_err(|e| {
        tracing::error!("DB Connection error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let results = blogs
        .filter(is_active.eq(1))
        .load::<Blog>(&mut conn)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch blogs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(results.into_iter().map(BlogDto::from).collect()))
}

#[utoipa::path(
    get,
    path = "/api/v1/blogs/{id}",
    params(
        ("id" = i32, Path, description = "Blog database id")
    ),
    responses(
        (status = 200, description = "A specific blog", body = BlogDto),
        (status = 404, description = "Blog not found"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("ApiKeyAuth" = [])
    )
)]
pub async fn get_blog_by_id(
    State(state): State<AppState>,
    Path(blog_id): Path<i32>,
) -> Result<Json<BlogDto>, StatusCode> {
    use crate::schema::blogs::dsl::*;

    let mut conn = state.get_conn().await.map_err(|e| {
        tracing::error!("DB Connection error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let blog = blogs
        .filter(id.eq(blog_id))
        .first::<Blog>(&mut conn)
        .await
        .map_err(|e| {
            if e == diesel::result::Error::NotFound {
                StatusCode::NOT_FOUND
            } else {
                tracing::error!("Failed to fetch blog: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(Json(BlogDto::from(blog)))
}
