use std::env;
use diesel::prelude::*;
use dotenvy::dotenv;

pub async fn establish_connection () -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url).expect("Unable to establish database connection")
}