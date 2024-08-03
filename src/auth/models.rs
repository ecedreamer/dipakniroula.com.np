use diesel::Insertable;


#[derive(Insertable)]
#[diesel(table_name = crate::schema::social_links)]
pub struct NewSocialLink<'a> {
    pub social_media: &'a str,
    pub social_link: &'a str,
}