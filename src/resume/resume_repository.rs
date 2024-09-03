use diesel::prelude::*;
use diesel::RunQueryDsl;
use crate::resume::models::{Experience, NewExperience, UpdateExperience};
use crate::schema::experiences::dsl::*;


pub struct ExperienceRepository<'a> {
    pub conn: &'a mut SqliteConnection,
}


impl<'a> ExperienceRepository<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    pub fn find(self) -> Result<Vec<Experience>, diesel::result::Error> {
        experiences
            .select(Experience::as_select())
            .order(id.desc())
            .load::<Experience>(self.conn)
    }
}