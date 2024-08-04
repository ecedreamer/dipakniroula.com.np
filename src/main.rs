mod route_handlers;
mod models;
mod schema;
mod db;
mod embedded_migrations;
mod auth;
mod blog;

use axum::{routing::get, Router};
use db::establish_connection;

use tower_http::services::ServeDir;

use diesel_migrations::MigrationHarness;

use time::Duration;

use route_handlers::{home_page, contact_page};

use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};


async fn handle_404() -> &'static str{
    "404 Page not found"
}


#[tokio::main]
async fn main() {

    tracing_subscriber::fmt::init();

    let session_store = MemoryStore::default();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::minutes(15)));


    let mut connection = establish_connection().await;
    connection.run_pending_migrations(embedded_migrations::MIGRATIONS)
        .expect("Error running migrations");


    let static_files_service = ServeDir::new("static");
    let app = Router::new()
        .route("/", get(home_page))
        .route("/contact", get(contact_page))
        .nest("/auth", auth::route_handlers::auth_routes().await)
        .nest("/blog", blog::route_handlers::blog_routes().await)
        .nest_service("/static", static_files_service)
        .fallback(handle_404)
        .layer(session_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("Server listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}


