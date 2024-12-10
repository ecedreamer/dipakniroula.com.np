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

    pub fn find_by_id(self, data_id: i32) -> QueryResult<Experience> {
        
        experiences
            .select(Experience::as_select())
            .filter(id.eq(data_id))
            .first::<Experience>(self.conn)
    }

    pub fn find(self) -> Result<Vec<Experience>, diesel::result::Error> {
        
        experiences
            .order(order.desc())
            .load::<Experience>(self.conn)
    }

    pub fn insert_one(self, data: &NewExperience) -> QueryResult<usize> {
        
        diesel::insert_into(experiences)
            .values(data)
            .execute(self.conn)
    }

    pub fn update_one(self, data_id: i32, data: &UpdateExperience) -> QueryResult<usize> {
        let target = experiences.filter(id.eq(data_id));
        
        diesel::update(target)
            .set(data)
            .execute(self.conn)
    }
}