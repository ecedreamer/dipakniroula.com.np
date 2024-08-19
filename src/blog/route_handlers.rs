use diesel::prelude::*;


use serde::Deserialize;


use askama::Template;
use axum::{http::StatusCode, response::{Html, IntoResponse, Redirect}, routing::{get, post}, Form, Router};
use axum::extract::{Multipart, Path};
use crate::db::establish_connection;
use crate::models::{Blog, NewBlog, UpdateBlog};
use diesel::RunQueryDsl;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::filter::{truncate_words, strip_tags};
use crate::middlewares::auth_middleware;
use std::str::FromStr;


pub async fn blog_routes() -> Router {
    Router::new()
        // client side pages
        .route("/list", get(blog_list_page))
        .route("/:id/detail", get(blog_detail_page))

        // admin side pages
        .route("/admin/list", get(blog_list_page_admin).layer(axum::middleware::from_fn(auth_middleware)))
        .route("/admin/create", get(blog_create_page).layer(axum::middleware::from_fn(auth_middleware)))
        .route("/admin/create", post(blog_create_handler).layer(axum::middleware::from_fn(auth_middleware)))
        .route("/admin/:blog_id/update", get(blog_update_page).layer(axum::middleware::from_fn(auth_middleware)))
        .route("/admin/:blog_id/update", post(blog_update_handler).layer(axum::middleware::from_fn(auth_middleware)))
        .route("/admin/:blog_id/delete", get(blog_delete_page).layer(axum::middleware::from_fn(auth_middleware)))
        .route("/admin/:blog_id/delete", post(blog_delete_handler).layer(axum::middleware::from_fn(auth_middleware)))
}


#[derive(Template, Deserialize)]
#[template(path = "admin/blogcreate.html")]
struct BlogCreateTemplate {
    page: String,
}

pub async fn blog_create_page() -> impl IntoResponse {
    let context = BlogCreateTemplate {
        page: "Blog Create".to_owned()
    };

    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        )
            .into_response()
    }
}


pub async fn blog_create_handler(mut multipart: Multipart) -> impl IntoResponse {
    tracing::info!("Handling multipart request");
    let mut image_path = String::new();
    let mut title = String::new();
    let mut content = String::new();
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap().to_string();
        tracing::info!("{}", field_name);

        if field_name == "blog-image" {
            let file_name = field.file_name().unwrap().to_string();


            if !file_name.is_empty() {
                image_path = format!("{}{}", "media/", file_name);
                let mut file = File::create(image_path.clone()).await.unwrap();
                while let Some(chunk) = field.chunk().await.unwrap() {
                    file.write_all(&chunk).await.unwrap();
                }
            }
        } else if field_name == "title" {
            title = field.text().await.unwrap();
        } else if field_name == "content" {
            content = field.text().await.unwrap();
        }
    }

    use crate::schema::blogs::dsl::blogs;

    let conn = &mut establish_connection().await;
    let blog = NewBlog {
        title: &title,
        content: &content,
        image: (!image_path.is_empty()).then_some(image_path.as_str()),
    };

    diesel::insert_into(blogs)
        .values(&blog)
        .execute(conn)
        .unwrap();

    Redirect::to("/blog/admin/list").into_response()
}


#[derive(Template, Deserialize)]
#[template(path = "admin/blogupdate.html")]
struct BlogUpdateTemplate {
    page: String,
    blog: Blog,
}

pub async fn blog_update_page(Path(blog_id): Path<String>) -> impl IntoResponse {
    use crate::schema::blogs::dsl::*;

    let mut conn = establish_connection().await;

    let blog_id_num = i32::from_str(&blog_id).unwrap();
    let result = blogs
        .filter(id.eq(blog_id_num))
        .limit(1)
        .first::<Blog>(&mut conn);
    match result {
        Ok(blog) => {
            let context = BlogUpdateTemplate {
                page: "Blog Update".to_owned(),
                blog: blog,
            };

            match context.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to render HTML".to_string(),
                )
                    .into_response()
            }
        }
        Err(e) => {
            Redirect::to("/blog/admin/list").into_response()
        }
    }
}


pub async fn blog_update_handler(Path(blog_id): Path<String>, mut multipart: Multipart) -> impl IntoResponse {
    use crate::schema::blogs::dsl::*;

    let mut update_blog = UpdateBlog {
        title: None,
        content: None,
        image: None,
    };

    let mut image_path = String::new();
    let mut new_title = String::new();
    let mut new_content = String::new();
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap().to_string();

        if field_name == "title" {
            new_title = field.text().await.unwrap_or(String::new());
            if !new_title.is_empty() {
                update_blog.title = Some(new_title);
            }
        } else if field_name == "content" {
            new_content = field.text().await.unwrap_or(String::new());
            if !new_content.is_empty() {
                update_blog.content = Some(new_content);
            }
        } else if field_name == "blog-image" {
            let file_name = field.file_name().unwrap().to_string();
            image_path = format!("{}{}", "media/", file_name);


            if !file_name.is_empty() {
                let mut file = File::create(image_path.clone()).await.unwrap();
                while let Some(chunk) = field.chunk().await.unwrap() {
                    file.write_all(&chunk).await.unwrap();
                }
                update_blog.image = Some(image_path);
            }
        }
    }


    let conn = &mut establish_connection().await;
    let blog_id_num = i32::from_str(&blog_id).unwrap();

    let target = blogs.filter(id.eq(blog_id_num));


    diesel::update(target)
        .set(&update_blog)
        .execute(conn)
        .unwrap();


    Redirect::to("/blog/admin/list").into_response()
}


#[derive(Template)]
#[template(path = "admin/blogdelete.html")]
struct BlogDeleteTemplate {
    page: String,
    blog_id: i32
}

async fn blog_delete_page(Path(blog_id): Path<String>) -> impl IntoResponse {
    let context = BlogDeleteTemplate {
        page: "Blog Delete".to_string(),
        blog_id: i32::from_str(&blog_id).unwrap()
    };

    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR, "Failed to render HTML".to_string()
        ).into_response()
    }

}


async fn blog_delete_handler(Path(blog_id): Path<String>) -> impl IntoResponse {
    let blog_id_num = i32::from_str(&blog_id).unwrap();
    let connection = &mut establish_connection().await;
    use crate::schema::blogs::dsl::*;

    diesel::delete(blogs.filter(id.eq(blog_id_num)))
        .execute(connection)
        .expect("Error deleting posts");
    Redirect::to("/blog/admin/list")
}


#[derive(Template)]
#[template(path = "blog_list.html")]
struct BlogListTemplate {
    blog_list: Vec<Blog>,
}


#[derive(Template)]
#[template(path = "admin/adminbloglist.html")]
struct AdminBlogListTemplate {
    blog_list: Vec<Blog>,
}


pub async fn blog_list_page_admin() -> impl IntoResponse {
    use crate::schema::blogs::dsl::*;

    let mut conn = establish_connection().await;

    let results = blogs
        .order(id.desc())
        .load::<Blog>(&mut conn)
        .expect("Error loading blogs");

    let context = crate::blog::route_handlers::AdminBlogListTemplate {
        blog_list: results.to_vec(),
    };

    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        )
            .into_response(),
    }
}



pub async fn blog_list_page() -> impl IntoResponse {
    use crate::schema::blogs::dsl::*;

    let mut conn = establish_connection().await;

    let results = blogs
        .order(id.desc())
        .load::<Blog>(&mut conn)
        .expect("Error loading blogs");

    let context = BlogListTemplate {
        blog_list: results.to_vec(),
    };

    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        )
            .into_response(),
    }
}


#[derive(Template)]
#[template(path = "blogdetail.html")]
struct BlogDetailTemplate {
    blog: Blog,
}


pub async fn blog_detail_page(Path(blog_id): Path<String>) -> impl IntoResponse {

    let mut conn = establish_connection().await;

    let blog_id_num = i32::from_str(&blog_id).unwrap();

    use crate::schema::blogs::dsl::*;

    let result = blogs
        .filter(id.eq(blog_id_num))
        .limit(1)
        .first::<Blog>(&mut conn);


    match result {
        Ok(blog) => {
            let context = BlogDetailTemplate {
                blog: blog,
            };

            match context.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to render HTML".to_string(),
                )
                    .into_response()
            }
        }
        Err(e) => {
            Redirect::to("/blog/list").into_response()
        }
    }
}