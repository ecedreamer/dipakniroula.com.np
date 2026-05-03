mod api;
mod auth;
mod blog;
mod db;
mod embedded_migrations;
mod middlewares;
mod models;
mod resume;
mod route_handlers;
mod schema;

mod filters;
mod session_backend;
mod state;
mod utils;

use argon2::password_hash::rand_core;
use askama::Template;
use axum::http::{HeaderValue, Method};
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    extract::DefaultBodyLimit,
};
use db::establish_sync_connection;
use dotenvy::dotenv;
use std::env;

use crate::db::create_pool;
pub use crate::state::AppState;
use axum_csrf::CsrfConfig;

use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use diesel_migrations::MigrationHarness;
use route_handlers::{contact_form_handler, contact_page, home_page, summernote_upload};
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Layer, filter, fmt};

#[derive(Template)]
#[template(path = "404.html")]
struct FourZeroFourTemplate {
    pub flash: Option<crate::models::FlashData>,
}

async fn handle_404() -> impl IntoResponse {
    tracing::warn!("404 page not found");
    let context = FourZeroFourTemplate { flash: None };
    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        )
            .into_response(),
    }
}

fn init_tracing() -> (
    tracing_appender::non_blocking::WorkerGuard,
    tracing_appender::non_blocking::WorkerGuard,
) {
    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(EnvFilter::new("trace"));

    let log_dir = env::var("LOG_DIRECTORY").expect("LOG_DIRECTORY not set");

    let app_file = tracing_appender::rolling::daily(&log_dir, "app.log");
    let error_file = tracing_appender::rolling::daily(&log_dir, "error.log");

    let (app_file_writer, _app_guard) = tracing_appender::non_blocking(app_file);
    let (error_file_writer, _err_guard) = tracing_appender::non_blocking(error_file);

    let app_layer = fmt::layer()
        .with_writer(app_file_writer)
        .with_line_number(true)
        .with_filter(filter::filter_fn(|metadata| {
            [&Level::DEBUG, &Level::INFO].contains(&metadata.level())
        }));
    let error_layer = fmt::layer()
        .with_writer(error_file_writer)
        .with_line_number(true)
        .with_thread_ids(true)
        .json()
        .with_filter(filter::LevelFilter::WARN);

    let subscriber = tracing_subscriber::registry()
        .with(console_layer)
        .with(app_layer)
        .with(error_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");

    (_app_guard, _err_guard)
}

async fn run_fixture(conn: &mut diesel::PgConnection) {
    tracing::info!("Running fixture script...");
    let env_email =
        std::env::var("WEB_SUPER_ADMIN").expect("WEB_SUPER_ADMIN not set in environment");
    let env_password = std::env::var("WEB_PASSWORD").expect("WEB_PASSWORD not set in environment");

    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString},
    };
    use diesel::prelude::*;

    let salt = SaltString::generate(&mut rand_core::OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(env_password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    use crate::schema::admin_users::dsl::*;

    let existing_user = admin_users
        .filter(email.eq(&env_email))
        .first::<crate::models::AdminUser>(conn);
    if existing_user.is_ok() {
        tracing::info!(
            "Admin user '{}' already exists. Skipping insertion.",
            env_email
        );
        return;
    }

    let insert_result = diesel::insert_into(admin_users)
        .values((email.eq(env_email.clone()), password.eq(hashed_password)))
        .execute(conn);

    match insert_result {
        Ok(_) => tracing::info!("Successfully inserted admin user: {}", env_email),
        Err(e) => tracing::error!("Failed to insert admin user: {}", e),
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let _guards = init_tracing();

    let csrf_config = CsrfConfig::default();
    let db_pool = create_pool();
    let app_state = AppState {
        db_pool,
        csrf_config: csrf_config.clone(),
    };

    let mut connection = establish_sync_connection();
    connection
        .run_pending_migrations(embedded_migrations::MIGRATIONS)
        .expect("Error running migrations");

    if std::env::args().any(|arg| arg == "--fixture") {
        run_fixture(&mut connection).await;
        return;
    }

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(
            "https://dipakniroula.com.np"
                .parse::<HeaderValue>()
                .expect("Invalid origin URL"),
        );

    let static_files_service = ServeDir::new("static");
    let media_files_service = ServeDir::new("media");

    let app = Router::new()
        .route("/", get(home_page))
        .route_service("/robots.txt", ServeFile::new("static/robots.txt"))
        .route("/contact", get(contact_page).post(contact_form_handler))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            middlewares::optional_session_middleware,
        ))
        .route("/summernote-upload", post(summernote_upload))
        .nest("/api", api::configure_api())
        .nest(
            "/auth",
            auth::route_handlers::auth_routes(app_state.clone()).await,
        )
        .merge(blog::route_handlers::blog_routes(app_state.clone()).await)
        .merge(resume::route_handlers::resume_routes(app_state.clone()).await)
        .nest_service("/static", static_files_service)
        .nest_service("/media", media_files_service)
        .fallback(handle_404)
        .with_state(app_state) // Now all routes that need state have it
        .layer(DefaultBodyLimit::max(20 * 1024 * 1024))
        .layer(axum::middleware::from_fn(
            middlewares::security_headers_middleware,
        ))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&"0.0.0.0:8081")
        .await
        .unwrap();
    tracing::debug!(
        "Server listening on http://{}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}
