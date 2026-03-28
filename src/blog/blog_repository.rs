use super::models::{Category, NewBlog, UpdateBlog};
use diesel::prelude::*;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;

pub mod blog_repo {
    use super::*;
    use crate::blog::models::{Blog, BlogCategory};
    use crate::schema::blog_categories;
    use crate::schema::blogs;

    pub struct BlogRepository<'a> {
        pub conn: &'a mut AsyncPgConnection,
    }

    impl<'a> BlogRepository<'a> {
        pub fn new(conn: &'a mut AsyncPgConnection) -> Self {
            Self { conn }
        }

        pub async fn find_by_id(self, data_id: i32) -> QueryResult<Blog> {
            blogs::dsl::blogs
                .filter(blogs::dsl::id.eq(data_id))
                .first::<Blog>(self.conn)
                .await
        }

        pub async fn find(self) -> Result<Vec<Blog>, diesel::result::Error> {
            blogs::dsl::blogs
                .order(blogs::dsl::id.desc())
                .load::<Blog>(self.conn)
                .await
        }

        pub async fn find_active_only(
            self,
            category_option: Option<i32>,
            order_by: &str,
            limit: i64,
        ) -> Result<Vec<Blog>, diesel::result::Error> {
            match category_option {
                Some(category_id) => {
                    blogs::dsl::blogs
                        .inner_join(
                            blog_categories::dsl::blog_categories
                                .on(blog_categories::dsl::blog_id.eq(blogs::id)),
                        )
                        .filter(blog_categories::category_id.eq(category_id))
                        .filter(blogs::is_active.eq(1))
                        .order(blogs::id.desc())
                        .select(blogs::all_columns) // Select all columns from blogs
                        .load::<Blog>(self.conn)
                        .await
                }
                None => match order_by {
                    "view_count" => {
                        blogs::dsl::blogs
                            .filter(blogs::is_active.eq(1))
                            .order(blogs::view_count.desc())
                            .limit(limit)
                            .load::<Blog>(self.conn)
                            .await
                    }
                    _ => {
                        blogs::dsl::blogs
                            .filter(blogs::is_active.eq(1))
                            .order(blogs::id.desc())
                            .limit(limit)
                            .load::<Blog>(self.conn)
                            .await
                    }
                },
            }
        }

        pub async fn increase_view_count(self, blog_id: i32) {
            let _ = diesel::update(blogs::dsl::blogs.filter(blogs::dsl::id.eq(blog_id)))
                .set(blogs::view_count.eq(blogs::view_count + 1))
                .execute(self.conn)
                .await;
        }

        pub async fn insert_one(self, data: &NewBlog<'_>, categories: &[i32]) {
            diesel::insert_into(blogs::dsl::blogs)
                .values(data)
                .execute(self.conn)
                .await
                .unwrap();

            let created_blog = blogs::dsl::blogs
                .order(blogs::dsl::id.desc())
                .first::<Blog>(self.conn)
                .await
                .unwrap();

            let blog_cat_data = BlogCategory {
                blog_id: created_blog.id.unwrap(),
                category_id: categories[0],
            };

            diesel::insert_into(blog_categories::dsl::blog_categories)
                .values(blog_cat_data)
                .execute(self.conn)
                .await
                .unwrap();
        }

        pub async fn update_one(self, blog_id: i32, data: &UpdateBlog) {
            let target = blogs::dsl::blogs.filter(blogs::dsl::id.eq(blog_id));

            diesel::update(target)
                .set(data)
                .execute(self.conn)
                .await
                .unwrap();
        }
    }
}

pub mod category_repository {
    use super::*;
    use crate::blog::models::NewCategory;
    use crate::schema::categories::dsl::*;

    pub struct CategoryRepository<'a> {
        pub conn: &'a mut AsyncPgConnection,
    }

    impl<'a> CategoryRepository<'a> {
        pub fn new(conn: &'a mut AsyncPgConnection) -> Self {
            Self { conn }
        }
        pub async fn find(self) -> Result<Vec<Category>, diesel::result::Error> {
            categories
                .order(id.desc())
                .load::<Category>(self.conn)
                .await
        }

        pub async fn insert_one(self, data: &NewCategory) {
            diesel::insert_into(categories)
                .values(data)
                .execute(self.conn)
                .await
                .unwrap();
        }
    }
}
