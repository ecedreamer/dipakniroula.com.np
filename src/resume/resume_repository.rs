use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::AsyncPgConnection;
use crate::resume::models::{Experience, NewExperience, UpdateExperience};
use crate::schema::experiences::dsl::*;

pub struct ExperienceRepository<'a> {
    pub conn: &'a mut AsyncPgConnection,
}

impl<'a> ExperienceRepository<'a> {
    pub fn new(conn: &'a mut AsyncPgConnection) -> Self {
        Self { conn }
    }

    pub async fn find_by_id(self, data_id: i32) -> QueryResult<Experience> {
        experiences
            .select(Experience::as_select())
            .filter(id.eq(data_id))
            .first::<Experience>(self.conn)
            .await
    }

    pub async fn find(self) -> Result<Vec<Experience>, diesel::result::Error> {
        experiences
            .order(order.asc())
            .load::<Experience>(self.conn)
            .await
    }

    pub async fn insert_one(self, data: &NewExperience) -> QueryResult<usize> {
        diesel::insert_into(experiences)
            .values(data)
            .execute(self.conn)
            .await
    }

    pub async fn update_one(self, data_id: i32, data: &UpdateExperience) -> QueryResult<usize> {
        let target = experiences.filter(id.eq(data_id));
        
        diesel::update(target)
            .set(data)
            .execute(self.conn)
            .await
    }
}