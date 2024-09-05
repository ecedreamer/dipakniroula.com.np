use diesel::prelude::*;
use diesel::{Queryable, Selectable};
use serde::{Deserialize, Serialize};


#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::experiences)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Experience {
    pub id: Option<i32>,
    pub company_name: String,
    pub your_position: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub responsibility: Option<String>,
    pub skills: Option<String>,
    pub company_link: String,
    pub order: i32
}


#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::experiences)]
pub struct NewExperience {
    pub company_name: String,
    pub company_link: String,
    pub your_position: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub responsibility: Option<String>,
    pub skills: Option<String>,
    pub order: i32
}


#[derive(Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::experiences)]
pub struct UpdateExperience {
    pub company_name: Option<String>,
    pub company_link: Option<String>,
    pub your_position: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub responsibility: Option<String>,
    pub skills: Option<String>,
    pub order: Option<i32>
}