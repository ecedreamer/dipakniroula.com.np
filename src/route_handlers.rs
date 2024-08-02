use std::collections::HashMap;
use askama::Template;
use diesel::prelude::*;
use diesel::RunQueryDsl;


use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use serde::Deserialize;
use serde_json::{from_str, Value};
use tokio::fs::read_to_string;
use crate::db::establish_connection;
use crate::models::{Blogs, SocialLink};

#[derive(Template, Deserialize)]
#[template(path = "home.html")]
struct HomeTemplate {
    name: String,
    current_company: String,
    current_position: String,
    company_link: String,
    skills: HashMap<String, Vec<String>>,
}

pub async fn home_page() -> impl IntoResponse {
    let json_str = read_to_string("profile.json").await.unwrap();
    let content: Value = from_str(&json_str).unwrap();

    let name = content["general_introduction"]["name"].as_str().unwrap().to_string();
    let current_company = content["general_introduction"]["current_company"].as_str().unwrap().to_string();
    let current_position = content["general_introduction"]["current_position"].as_str().unwrap().to_string();
    let company_link = content["general_introduction"]["company_link"].as_str().unwrap().to_string();
    let skills= content["skills"].as_object().unwrap().iter()
        .map(|(k, v)| {
            (k.clone(), v.as_array().unwrap().iter().map(|s| s.as_str().unwrap().to_string()).collect())
        })
        .collect();


    let context = HomeTemplate {
        name,
        current_company,
        current_position,
        company_link,
        skills,
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


struct Blog {
    title: String,
    content: String,
}


#[derive(Template)]
#[template(path = "blog_list.html")]
struct BlogListTemplate {
    blog_list: Vec<Blogs>,
}


pub async fn blog_list_page() -> impl IntoResponse {

    use crate::schema::blogs::dsl::*;

    let mut conn = establish_connection().await;

    let results = blogs
        .load::<Blogs>(&mut conn)
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
#[template(path = "contact.html")]
struct ContactTemplate {
    social_links: Vec<SocialLink>
}


pub async fn contact_page() -> impl IntoResponse {

    let connection = &mut establish_connection().await;
    use crate::schema::social_links::dsl::social_links;
    let results = social_links
        .select(SocialLink::as_select())
        .load(connection)
        .expect("Error loading posts");

        println!("Displaying {} posts", results.len());
        // for social_link in results {
        //     println!("{}", social_link.social_media);
        //     println!("-----------\n");
        //     println!("{}", social_link.social_link);
        // }

    let context = ContactTemplate {
        social_links: results
    };
    match context.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render HTML".to_string(),
        ).into_response(),
    }
}

