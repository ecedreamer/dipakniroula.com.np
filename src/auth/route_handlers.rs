use serde::Deserialize;

use askama::Template;
use axum::{http::StatusCode, response::{Html, IntoResponse, Redirect}, routing::{get, post}, Form, Router};





pub async fn auth_routes() -> Router {
    Router::new()
    .route("/login", get(login_page))
    .route("/login", post(login_handler))
    .route("/admin-panel", get(admin_home_page))
}


#[derive(Template, Deserialize)]
#[template(path = "login.html")]
struct LoginTemplate {
    page: String
}

pub async fn login_page() -> impl IntoResponse {
    let context = LoginTemplate {
        page: "Login".to_owned()
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


#[derive(Deserialize, Debug)]
struct LoginForm {
    email: String,
    password: String,
}


pub async fn login_handler(Form(form_data): Form<LoginForm>) -> impl IntoResponse {

    tracing::info!("Hello, {}! Your email is {}", form_data.email, form_data.password);
    if form_data.email == "sangit.niroula@gmail.com".to_string() && form_data.password == "admin@123".to_string() {
        Redirect::to("/auth/admin-panel")
    } else{
        Redirect::to("/auth/login")
    }
}


#[derive(Template, Deserialize)]
#[template(path = "admin_home.html")]
struct AdminHomeTemplate {
    page: String
}

pub async fn admin_home_page() -> impl IntoResponse {
    let context = AdminHomeTemplate {
        page: "Admin Home".to_owned()
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