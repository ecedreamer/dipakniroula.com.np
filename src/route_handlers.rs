use std::collections::HashMap;
use indexmap::IndexMap;
use askama::Template;
use axum::Form;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use chrono::Utc;


use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum_csrf::CsrfToken;
use serde::Deserialize;
use serde_json::{from_str, Map, Value};
use tokio::fs::read_to_string;
use crate::db::establish_connection;
use crate::models::SocialLink;


#[derive(Template, Deserialize)]
#[template(path = "home.html")]
struct HomeTemplate {
    name: String,
    current_company: String,
    current_position: String,
    company_link: String,
    social_links: Vec<SocialLink>,
}

pub async fn home_page() -> impl IntoResponse {
    let json_str = read_to_string("profile.json").await.unwrap();
    let content: Value = from_str(&json_str).unwrap();

    let connection = &mut establish_connection().await;

    let name = content["general_introduction"]["name"].as_str().unwrap().to_string();
    let current_company = content["general_introduction"]["current_company"].as_str().unwrap().to_string();
    let current_position = content["general_introduction"]["current_position"].as_str().unwrap().to_string();
    let company_link = content["general_introduction"]["company_link"].as_str().unwrap().to_string();

    use crate::schema::social_links::dsl::social_links;
    let my_social_links = social_links
        .select(SocialLink::as_select())
        .load(connection)
        .expect("Error loading social links");


    let context = HomeTemplate {
        name,
        current_company,
        current_position,
        company_link,
        social_links: my_social_links,
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
#[template(path = "contact.html")]
struct ContactTemplate {
    social_links: Vec<SocialLink>,
    authenticity_token: String,
}


pub async fn contact_page(token: CsrfToken) -> impl IntoResponse {
    let connection = &mut establish_connection().await;
    use crate::schema::social_links::dsl::social_links;
    let results = social_links
        .select(SocialLink::as_select())
        .load(connection)
        .expect("Error loading social links");


    let context = ContactTemplate {
        social_links: results,
        authenticity_token: token.authenticity_token().unwrap(),
    };
    match context.render() {
        Ok(html) => (token, Html(html)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        ).into_response(),
    }
}


#[derive(Deserialize, Debug)]
pub struct ContactForm {
    authenticity_token: String,
    full_name: String,
    email: String,
    mobile: Option<String>,
    subject: String,
    message: String,
}

pub async fn contact_form_handler(token: CsrfToken, Form(form): Form<ContactForm>) -> impl IntoResponse {
    tracing::info!("{} ------", form.full_name);
    let date_time_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mobile = form.mobile.unwrap_or(String::new());

    if token.verify(&form.authenticity_token).is_err() {
        tracing::error!("Suspicious Payload: {}", &form.authenticity_token);
        Redirect::to("/contact").into_response()
    } else {
        use crate::schema::messages;

        let contact_message = crate::models::NewContactMessage {
            full_name: form.full_name.as_str(),
            email: form.email.as_str(),
            mobile: Some(mobile.as_str()),
            subject: form.subject.as_str(),
            message: form.message.as_str(),
            date_sent: date_time_str.as_str(),
        };
        let conn = &mut establish_connection().await;

        diesel::insert_into(messages::table)
            .values(&contact_message)
            .execute(conn)
            .unwrap();

        Redirect::to("/").into_response()
    }
}


