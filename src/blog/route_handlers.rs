use diesel::prelude::*;

use serde::Deserialize;

use super::models::{Blog, Category, NewBlog, NewCategory, UpdateBlog};
use crate::blog::blog_repository::blog_repo::BlogRepository;
use crate::blog::blog_repository::category_repository::CategoryRepository;

use crate::middlewares::session_middleware;
use crate::filters;
use crate::state::AppState;
use crate::utils::error::AppError;
use std::str::FromStr;
use askama::Template;
use axum::extract::{Multipart, Path, Query, State};
use axum::{
    Form, Router,
    response::{Html, IntoResponse, Redirect},
    routing::get,
};
use chrono::Utc;
use diesel_async::RunQueryDsl;

use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn blog_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // client side pages
        .route("/blog/list", get(blog_list_page))
        .route(
            "/admin/blog/create",
            get(blog_create_page)
                .post(blog_create_handler)
                .layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
        .route(
            "/admin/blog/{blog_id}/update",
            get(blog_update_page)
                .post(blog_update_handler)
                .layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
        .route(
            "/admin/blog/{blog_id}/delete",
            get(blog_delete_page)
                .post(blog_delete_handler)
                .layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
        .route(
            "/admin/blog/list",
            get(blog_list_page_admin).layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
        .route(
            "/admin/category/create",
            get(category_create_page)
                .post(category_create_handler)
                .layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
        .route(
            "/admin/category/list",
            get(category_list_page).layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
        .route(
            "/admin/category/{category_id}/update",
            get(category_update_page)
                .post(category_update_handler)
                .layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
        .route(
            "/admin/category/{category_id}/delete",
            get(category_delete_handler).layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
        .route("/blogs", get(blog_list_page))
        .route("/blog/{blog_id}/detail", get(blog_detail_page))
}



#[derive(Template)]
#[template(path = "admin/blogcreate.html")]
struct BlogCreateTemplate {
    categories: Vec<Category>,
    active_nav: String,
}



pub async fn blog_create_page(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    let category_repo = CategoryRepository::new(&mut conn);
    let context = BlogCreateTemplate {
        categories: category_repo.find().await?,
        active_nav: "blogs".to_string(),
    };

    Ok(Html(context.render()?))
}

pub async fn blog_create_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("Handling multipart request");
    let mut image_path = String::new();
    let mut title = String::new();
    let mut content = String::new();
    let mut blog_status = 0;
    let mut categories = Vec::new();
    
    while let Ok(Some(mut field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();
        tracing::info!("{}", field_name);

        if field_name == "blog-image" {
            let file_name = field.file_name().unwrap_or("").to_string();

            if !file_name.is_empty() {
                let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
                image_path = format!("{}{}_{}", "media/", timestamp, file_name);
                let mut file = File::create(image_path.clone()).await?;
                while let Ok(Some(chunk)) = field.chunk().await {
                    file.write_all(&chunk).await?;
                }
            }
        } else if field_name == "title" {
            title = field.text().await.map_err(|e| AppError::Internal(e.to_string()))?;
        } else if field_name == "category" {
            let cat_id_str = field.text().await.map_err(|e| AppError::Internal(e.to_string()))?;
            if let Ok(cat_id) = i32::from_str(&cat_id_str) {
                categories.push(cat_id);
            }
        } else if field_name == "content" {
            content = field.text().await.map_err(|e| AppError::Internal(e.to_string()))?;
        } else if field_name == "is_active" {
            let value = field.text().await.map_err(|e| AppError::Internal(e.to_string()))?;
            blog_status = if value == "on" { 1 } else { 0 };
        }
    }

    let blog = NewBlog {
        is_active: blog_status,
        title: &title,
        content: &content,
        image: (!image_path.is_empty()).then_some(image_path.as_str()),
        published_date: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        modified_date: None,
    };

    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    let blog_repo = BlogRepository::new(&mut conn);

    blog_repo.insert_one(&blog, &categories).await?;

    Ok(Redirect::to("/admin/blog/list"))
}

#[derive(Template, Deserialize)]
#[template(path = "admin/blogupdate.html")]
struct BlogUpdateTemplate {
    blog: Blog,
    active_nav: String,
}

pub async fn blog_update_page(
    State(state): State<AppState>,
    Path(blog_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    use crate::schema::blogs::dsl::*;

    let mut conn: crate::db::PooledConn = state.get_conn().await?;

    let result = blogs
        .filter(id.eq(blog_id))
        .limit(1)
        .first::<Blog>(&mut conn)
        .await;
    
    match result {
        Ok(blog) => {
            let context = BlogUpdateTemplate { 
                blog,
                active_nav: "blogs".to_string(),
            };

            Ok(Html(context.render()?).into_response())
        }
        Err(_) => Ok(Redirect::to("/admin/blog/list").into_response()),
    }
}

pub async fn blog_update_handler(
    State(state): State<AppState>,
    Path(blog_id): Path<i32>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut update_blog = UpdateBlog {
        title: None,
        content: None,
        image: None,
        modified_date: None,
        is_active: Some(0),
        view_count: None,
    };

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "title" {
            let new_title = field.text().await.unwrap_or(String::new());
            if !new_title.is_empty() {
                update_blog.title = Some(new_title);
            }
        } else if field_name == "content" {
            let new_content = field.text().await.unwrap_or(String::new());
            if !new_content.is_empty() {
                update_blog.content = Some(new_content);
            }
        } else if field_name == "blog-image" {
            let file_name = field.file_name().unwrap_or("").to_string();
            let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
            let image_path = format!("{}{}_{}", "media/summernote/", timestamp, file_name);

            if !file_name.is_empty() {
                let mut file = File::create(image_path.clone()).await?;
                while let Ok(Some(chunk)) = field.chunk().await {
                    file.write_all(&chunk).await?;
                }
                update_blog.image = Some(image_path);
            }
        } else if field_name == "is_active" {
            let value = field.text().await.map_err(|e| AppError::Internal(e.to_string()))?;
            tracing::info!("This field is present: {}", value);

            if value == "on" {
                update_blog.is_active = Some(1);
            }
        }
    }
    update_blog.modified_date = Some(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());

    let mut conn: crate::db::PooledConn = state.get_conn().await?;

    let blog_repo = BlogRepository::new(&mut conn);
    blog_repo.update_one(blog_id, &update_blog).await?;

    Ok(Redirect::to("/admin/blog/list"))
}


#[derive(Template)]
#[template(path = "admin/blogdelete.html")]
struct BlogDeleteTemplate {
    blog_id: i32,
    active_nav: String,
}

async fn blog_delete_page(Path(blog_id): Path<i32>) -> Result<impl IntoResponse, AppError> {
    let context = BlogDeleteTemplate {
        blog_id,
        active_nav: "blogs".to_string(),
    };

    Ok(Html(context.render()?))
}

async fn blog_delete_handler(
    State(state): State<AppState>,
    Path(blog_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    use crate::schema::blogs::dsl::*;

    diesel::delete(blogs.filter(id.eq(blog_id)))
        .execute(&mut conn)
        .await?;
    
    Ok(Redirect::to("/admin/blog/list"))
}

#[derive(Deserialize, Debug)]
pub struct AdminBlogQuery {
    pub page: Option<i64>,
    pub query: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/adminbloglist.html")]
struct AdminBlogListTemplate {
    blog_list: Vec<Blog>,
    active_nav: String,
    current_page: i64,
    total_pages: i64,
    pages: Vec<i64>,
    search_query: Option<String>,
    total_count: i64,
}

pub async fn blog_list_page_admin(
    State(state): State<AppState>,
    Query(pagination): Query<AdminBlogQuery>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;

    let page = pagination.page.unwrap_or(1).max(1);
    let per_page = 10;
    let offset = (page - 1) * per_page;

    use crate::schema::blogs::dsl::*;

    let mut blogs_query = blogs.into_boxed();
    let mut count_query = blogs.into_boxed();

    if let Some(ref q) = pagination.query {
        if !q.trim().is_empty() {
            let search_term = format!("%{}%", q);
            blogs_query = blogs_query.filter(
                title.ilike(search_term.clone())
                    .or(content.ilike(search_term.clone()))
            );
            count_query = count_query.filter(
                title.ilike(search_term.clone())
                    .or(content.ilike(search_term))
            );
        }
    }

    let b_count = count_query.count().get_result::<i64>(&mut conn).await.unwrap_or(0);
    let results = blogs_query
        .order(id.desc())
        .limit(per_page)
        .offset(offset)
        .load::<Blog>(&mut conn)
        .await?;

    let total_pages = if b_count == 0 { 1 } else { (b_count + per_page - 1) / per_page };
    let pages_vec = (1..=total_pages).collect();

    let context = AdminBlogListTemplate { 
        blog_list: results,
        active_nav: "blogs".to_string(),
        current_page: page,
        total_pages,
        pages: pages_vec,
        search_query: pagination.query,
        total_count: b_count,
    };
    
    Ok(Html(context.render()?))
}

#[derive(Template)]
#[template(path = "blog_list.html")]
struct BlogListTemplate {
    blog_list: Vec<Blog>,
    categories_list: Vec<Category>,
}

#[derive(Debug, Deserialize)]
pub struct BlogQuery {
    cat_id: Option<i32>,
}

pub async fn blog_list_page(
    State(state): State<AppState>,
    Query(query): Query<BlogQuery>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    let blog_repo = BlogRepository::new(&mut conn);

    let results = blog_repo.find_active_only(query.cat_id, "id", 25).await?;

    let category_repo = CategoryRepository::new(&mut conn);
    let categories = category_repo.find().await?;

    let context = BlogListTemplate {
        blog_list: results,
        categories_list: categories,
    };

    Ok(Html(context.render()?))
}

#[derive(Template)]
#[template(path = "blogdetail.html")]
struct BlogDetailTemplate {
    blog: Blog,
}

pub async fn blog_detail_page(
    State(state): State<AppState>,
    Path(blog_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;

    let blog_repo = BlogRepository::new(&mut conn);
    let single_blog_result = blog_repo.find_by_id(blog_id).await;

    let blog_repo = BlogRepository::new(&mut conn);
    let _ = blog_repo.increase_view_count(blog_id).await;

    match single_blog_result {
        Ok(blog) => {
            let context = BlogDetailTemplate { blog };
            Ok(Html(context.render()?).into_response())
        }
        Err(err) => {
            tracing::warn!(
                "Blog with the id: {} does not exist; error: {:?}",
                blog_id,
                err
            );
            Ok(Redirect::to("/blog/list").into_response())
        }
    }
}

#[derive(Template)]
#[template(path = "admin/admincategorycreate.html")]
struct CategoryCreatePageTemplate {
    active_nav: String,
}

pub async fn category_create_page() -> Result<impl IntoResponse, AppError> {
    let context = CategoryCreatePageTemplate {
        active_nav: "categories".to_string(),
    };

    Ok(Html(context.render()?))
}

#[derive(Debug, Deserialize)]
pub struct CategoryForm {
    name: String,
}

pub async fn category_create_handler(
    State(state): State<AppState>,
    Form(form_data): Form<CategoryForm>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    let repo = CategoryRepository::new(&mut conn);

    let new_category = NewCategory {
        name: form_data.name,
    };

    repo.insert_one(&new_category).await?;

    Ok(Redirect::to("/admin/category/list"))
}

#[derive(Template)]
#[template(path = "admin/admincategorylist.html")]
struct CategoryListTemplate {
    categories: Vec<Category>,
    active_nav: String,
}

pub async fn category_list_page(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    let category_repo = CategoryRepository::new(&mut conn);

    let categories = category_repo.find().await?;

    let context = CategoryListTemplate {
        categories,
        active_nav: "categories".to_string(),
    };

    Ok(Html(context.render()?))
}

#[derive(Template)]
#[template(path = "admin/admincategoryupdate.html")]
struct CategoryUpdateTemplate {
    category: Category,
    active_nav: String,
}

pub async fn category_update_page(
    State(state): State<AppState>,
    Path(category_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    let category_repo = CategoryRepository::new(&mut conn);

    let category = category_repo.find_by_id(category_id).await?;

    let context = CategoryUpdateTemplate {
        category,
        active_nav: "categories".to_string(),
    };

    Ok(Html(context.render()?))
}

pub async fn category_update_handler(
    State(state): State<AppState>,
    Path(category_id): Path<i32>,
    Form(form_data): Form<CategoryForm>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    let category_repo = CategoryRepository::new(&mut conn);

    let update_category = NewCategory {
        name: form_data.name,
    };

    category_repo.update_one(category_id, &update_category).await?;

    Ok(Redirect::to("/admin/category/list"))
}

pub async fn category_delete_handler(
    State(state): State<AppState>,
    Path(category_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;
    let category_repo = CategoryRepository::new(&mut conn);

    category_repo.delete_one(category_id).await?;

    Ok(Redirect::to("/admin/category/list"))
}

