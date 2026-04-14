use std::env;
use diesel::prelude::*;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;

pub type DbPool = Pool<AsyncPgConnection>;
pub type PooledConn = diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>;

pub fn create_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder(config)
        .build()
        .expect("Could not build connection pool")
}


pub fn establish_sync_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect("Unable to establish sync database connection")
}