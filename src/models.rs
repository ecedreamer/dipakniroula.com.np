use diesel::prelude::*;
use serde::{Deserialize, Serialize};


#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct Blogs {
    #[diesel(sql_type = Nullable<Integer>)]
    pub id: Option<i32>,
    #[diesel(sql_type = Text)]
    pub title: String,
    #[diesel(sql_type = Text)]
    pub content: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::blogs)]
pub struct NewBlog<'a> {
    pub title: &'a str,
    pub content: &'a str,
}


#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::experiences)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Experience {
    pub id: Option<i32>,
    pub company_name: String,
    pub position: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub responsibility: Option<String>,

}


#[derive(Queryable, Selectable, Debug, Deserialize)]
#[diesel(table_name = crate::schema::social_links)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SocialLink {
    pub id: Option<i32>,
    pub social_media: String,
    pub social_link: String,
}