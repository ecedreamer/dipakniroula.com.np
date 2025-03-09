mod route_handlers;
mod models;
mod schema;
mod db;
mod embedded_migrations;
mod auth;
mod blog;
mod middlewares;
mod resume;

mod filters;
mod session_backend;

use std::env;
use std::sync::{Arc, Mutex};
use dotenvy::dotenv;
use askama::Template;
use axum::{
    routing::{get, post},
    Router,
    response::{Html, IntoResponse},
    http::StatusCode,
};
use axum::body::Body;
use axum::http::{HeaderValue, Method};
use axum::response::Response;
use db::establish_connection;

use axum_csrf::CsrfConfig;

use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;

use diesel_migrations::MigrationHarness;
use lazy_static::lazy_static;
use opentelemetry::global;
use rand::Rng;
use tracing::{span, Level};
use tracing_subscriber::{filter, fmt, EnvFilter, Layer, Registry};
use tracing_subscriber::layer::SubscriberExt;
use route_handlers::{
    home_page,
    contact_page,
    contact_form_handler,
    summernote_upload,
};


#[derive(Template)]
#[template(path = "404.html")]
struct FourZeroFourTemplate {}

async fn handle_404() -> impl IntoResponse {
    tracing::error!("This is a error message for error page");
    let otel_tracer = global::tracer("otel_tracer");
    let otel_tracer_span = otel_tracer.start("This is the otel tracer");
    tracing::error!("This is a error message for error page new tracer");
    let context = FourZeroFourTemplate {};
    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        )
            .into_response(),
    }
}


use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry::trace::{Tracer, TracerProvider as _};
use tracing_opentelemetry::MetricsLayer;

fn init_tracing() -> (tracing_appender::non_blocking::WorkerGuard, tracing_appender::non_blocking::WorkerGuard) {

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

    // otel configuration
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    opentelemetry::global::set_tracer_provider(provider.clone());

    let tracer = provider.tracer("otel_tracer");

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);


    let subscriber = tracing_subscriber::registry()
        .with(console_layer)
        .with(app_layer)
        .with(error_layer)
        .with(telemetry_layer)
        ;

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global subscriber");

    (_app_guard, _err_guard)
}


#[tokio::main]
async fn main() {
    dotenv().ok();

    let _guards = init_tracing();


    let csrf_config = CsrfConfig::default();

    let mut connection = establish_connection().await;
    connection.run_pending_migrations(embedded_migrations::MIGRATIONS)
        .expect("Error running migrations");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin("https://dipakniroula.com.np".parse::<HeaderValue>().expect("Invalid origin URL"));

    let static_files_service = ServeDir::new("static");
    let media_files_service = ServeDir::new("media");
    let app = Router::new()
        .route("/", get(home_page))
        .route("/contact", get(contact_page).post(contact_form_handler))
        .with_state(csrf_config)
        .route("/summernote-upload", post(summernote_upload))
        .nest("/auth", auth::route_handlers::auth_routes().await)
        .merge(blog::route_handlers::blog_routes().await)
        .merge(resume::route_handlers::resume_routes().await)
        .nest_service("/static", static_files_service)
        .nest_service("/media", media_files_service)
        .fallback(handle_404)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&"0.0.0.0:8080").await.unwrap();
    tracing::debug!("Server listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
