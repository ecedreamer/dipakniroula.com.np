use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime};



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


#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::sessions)]
pub struct CustomSession {
    pub id: Option<i32>,
    pub session_id: String,
    pub user_id: String,
    pub data: Option<String>,
    pub expires_at: NaiveDateTime,
}

