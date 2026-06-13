use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::oauth_users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OAuthUser {
    pub id: i32,
    pub google_id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::oauth_users)]
pub struct NewOAuthUser<'a> {
    pub google_id: &'a str,
    pub email: &'a str,
    pub name: &'a str,
    pub avatar_url: Option<&'a str>,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::quiz_attempts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct QuizAttemptWithUser {
    pub id: Option<i32>,
    pub player_name: String,
    pub player_email: String,
    pub topic: Option<String>,
    pub difficulty: String,
    pub num_questions: i32,
    pub score: i32,
    pub total_questions: i32,
    pub answers_json: Option<serde_json::Value>,
    pub played_at: NaiveDateTime,
    pub oauth_user_id: Option<i32>,
}
