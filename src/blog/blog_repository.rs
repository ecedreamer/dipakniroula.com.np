use diesel::prelude::*;
use diesel::RunQueryDsl;
use crate::models::Blog;
use crate::schema::blogs::dsl::*;


pub struct BlogRepository<'a> {
    pub conn: &'a mut SqliteConnection,
}


impl<'a> BlogRepository<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    pub fn find_by_id(self, blog_id: i32) -> QueryResult<Blog> {
        let result = blogs
            .filter(id.eq(blog_id))
            .first::<Blog>(self.conn);
        result
    }

    pub fn find(self) -> Result<Vec<Blog>, diesel::result::Error> {
        blogs
        .order(id.desc())
        .load::<Blog>(self.conn)
    }

    pub fn increase_view_count(self, blog_id: i32) {
        let _ = diesel::update(blogs.filter(id.eq(blog_id)))
        .set(view_count.eq(view_count + 1))
        .execute(self.conn);
    }
}
