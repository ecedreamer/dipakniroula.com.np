use diesel::prelude::*;
use diesel::{Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Message {
    pub id: Option<i32>,
    pub full_name: String,
    pub email: String,
    pub mobile: Option<String>,
    pub subject: String,
    pub message: String,
    pub date_sent: String,
