use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use serde::Deserialize;
use std::fs;

#[path = "../db.rs"]
mod db;
#[path = "../schema.rs"]
mod schema;

#[derive(diesel::Insertable, Deserialize)]
#[diesel(table_name = schema::experiences)]
pub struct NewExperience {
    pub company_name: String,
    pub company_link: String,
    pub your_position: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub responsibility: Option<String>,
    pub skills: Option<String>,
    pub order: i32
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    
    // We need to use localhost since we are running on the host, but the .env has 'db'
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let database_url = database_url.replace("@db:", "@localhost:");
    
    println!("Connecting to database at {}...", database_url);
    let mut conn = AsyncPgConnection::establish(&database_url).await?;

    println!("Loading experiences from JSON...");
    let file_content = fs::read_to_string("src/resume/experiences.json")?;
    let experiences: Vec<NewExperience> = serde_json::from_str(&file_content)?;

    println!("Cleaning up existing experiences...");
    diesel::delete(schema::experiences::table).execute(&mut conn).await?;

    println!("Importing {} experiences...", experiences.len());
    diesel::insert_into(schema::experiences::table)
        .values(&experiences)
        .execute(&mut conn)
        .await?;

    println!("Successfully imported experiences!");
    Ok(())
}
