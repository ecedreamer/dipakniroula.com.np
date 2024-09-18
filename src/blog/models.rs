use diesel::{AsChangeset, Insertable, Queryable, Selectable};
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


#[derive(Queryable, Deserialize, Debug)]
#[diesel(table_name = crate::schema::categories)]
pub struct Category {
    pub id: i32,
    pub name: String,
}

// Join table for the many-to-many relationship between blogs and tags
#[derive(Queryable, Insertable, Debug)]
#[diesel(table_name = crate::schema::blog_categories)]
pub struct BlogCategory {
    pub blog_id: i32,
    pub category_id: i32,
}


#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::categories)]
pub struct NewCategory {
    pub name: String
}