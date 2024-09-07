use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use serde::Deserialize;

#[derive(Queryable, Selectable, Debug, Deserialize)]
#[diesel(table_name = crate::schema::social_links)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SocialLink {
    pub id: Option<i32>,
    pub social_media: String,
    pub social_link: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::social_links)]
pub struct NewSocialLink<'a> {
    pub social_media: &'a str,
    pub social_link: &'a str,
}

#[derive(Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::social_links)]
pub struct UpdateSocialLink {
    pub social_media: Option<String>,
    pub social_link: Option<String>,
}
