use std::env;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection};

pub async fn establish_connection() -> AsyncPgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    AsyncPgConnection::establish(&database_url).await.expect("Unable to establish database connection")
}

pub fn establish_sync_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect("Unable to establish sync database connection")
}