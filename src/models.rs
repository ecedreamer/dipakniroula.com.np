use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::blogs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Blog {
    pub id: Option<i32>,
    pub title: String,
    pub content: String,
    pub image: Option<String>,
    pub published_date: String,
    pub modified_date: Option<String>,
    pub view_count: i32,
    pub is_active: i32,
}


#[derive(Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::blogs)]
pub struct UpdateBlog {
    pub is_active: Option<i32>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub image: Option<String>,
    pub modified_date: Option<String>,
    pub view_count: Option<i32>,
}


#[derive(Insertable)]
#[diesel(table_name = crate::schema::blogs)]
pub struct NewBlog<'a> {
    pub title: &'a str,
    pub content: &'a str,
    pub image: Option<&'a str>,
    pub is_active: i32,
    pub published_date: String,
    pub modified_date: Option<String>,
}


#[derive(Queryable, Selectable, Debug, Deserialize)]
#[diesel(table_name = crate::schema::social_links)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SocialLink {
    pub id: Option<i32>,
    pub social_media: String,
    pub social_link: String,
}


#[derive(Insertable)]
#[diesel(table_name = crate::schema::messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewContactMessage<'a> {
    pub full_name: &'a str,
    pub email: &'a str,
    pub mobile: Option<&'a str>,
    pub subject: &'a str,
    pub message: &'a str,
    pub date_sent: &'a str
}


#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ContactMessage {
    pub id: Option<i32>,
    pub full_name: String,
    pub email: String,
    pub mobile: Option<String>,
    pub subject: String,
    pub message: String,
    pub date_sent: String,
}


#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::admin_users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AdminUser {
    pub id: i32,
    pub email: String,
    pub password: String
}