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
use prometheus::{Encoder, Gauge, Registry, TextEncoder};
use rand::Rng;
use tracing::Level;
use tracing_subscriber::{filter, fmt, EnvFilter, Layer};
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


#[tokio::main]
async fn main() {
    dotenv().ok();


    // Tracing Configuration Start
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
        .with_filter(filter::filter_fn(|metadata| {
            [&Level::DEBUG, &Level::INFO].contains(&metadata.level())
        }));
    let error_layer = fmt::layer()
        .with_writer(error_file_writer)
        .with_filter(filter::LevelFilter::WARN);

    let subscriber = tracing_subscriber::registry()
        .with(console_layer)
        .with(app_layer)
        .with(error_layer)
        ;

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global subscriber");

    // Tracing Configuration End

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
        .route("/metrics", get(prom_metrics_handler))
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


pub fn collect_metrics() -> String {
    r#"
        # HELP http_requests_total Total number of HTTP requests
        # TYPE http_requests_total counter
        http_requests_total 10

        # HELP http_requests_by_status Number of HTTP requests by status code
        # TYPE http_requests_by_status counter
        http_requests_by_status{status="200"} 50

        # HELP current_active_users Number of active users
        # TYPE current_active_users gauge
        current_active_users 30

        go_threads{instance="localhost:9090", job="prometheus"}
    "#
        .to_string()
}

async fn prom_metrics_handler() -> impl IntoResponse {
    // Generate dynamic data for metrics
    let mut rng = rand::rng();
    let status_code_200 = rng.random_range(0..300); // Random status 200 requests count between 0 and 300

    // Lock the mutex to update the metric value
    let mut requests_by_status = REQUESTS_BY_STATUS.lock().unwrap();
    requests_by_status.set(status_code_200 as f64); // Update the metric with the new value

    // Prepare Prometheus response
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    let registry = Registry::new();

    // Register the metric in the registry
    registry.register(Box::new(requests_by_status.clone())).unwrap();

    encoder.encode(&registry.gather(), &mut buffer).unwrap();

    // Return the response with metrics as body
    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(Body::from(buffer))
        .unwrap()
}

lazy_static::lazy_static! {
    static ref REQUESTS_BY_STATUS: Arc<Mutex<Gauge>> = Arc::new(Mutex::new(Gauge::new("http_requests_by_status", "Number of HTTP requests by status code").unwrap()));
}