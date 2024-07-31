mod route_handlers;
mod models;
mod schema;
mod db;
mod embedded_migrations;

use axum::{routing::get, Router};
use db::establish_connection;

use tower_http::services::ServeDir;

use diesel_migrations::MigrationHarness;


use route_handlers::{home_page, blog_list_page, contact_page};




#[tokio::main]
async fn main() {

    tracing_subscriber::fmt::init();


    let mut connection = establish_connection().await;
    connection.run_pending_migrations(embedded_migrations::MIGRATIONS)
        .expect("Error running migrations");


    let static_files_service = ServeDir::new("static");
    let app = Router::new()
        .route("/", get(home_page))
        .route("/blogs/", get(blog_list_page))
        .route("/contact/", get(contact_page))
        .nest_service("/static", static_files_service);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("Server listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

