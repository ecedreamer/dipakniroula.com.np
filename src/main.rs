mod tracing_init;
mod route_handlers;

use axum::{routing::get, Router};


use tower_http::services::ServeDir;


use route_handlers::{home_page, blog_list_page, contact_page};


#[tokio::main]
async fn main() {

    tracing_subscriber::fmt::init();

    let static_files_service = ServeDir::new("static");
    let app = Router::new()
        .route("/", get(home_page))
        .route("/blogs/", get(blog_list_page))
        .route("/contact/", get(contact_page))
        .nest_service("/static", static_files_service);
    // .fallback_service(ServiceBuilder::new().service(static_files_service));;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("Server listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

