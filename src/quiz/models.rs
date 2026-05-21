use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::quiz_attempts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct QuizAttempt {
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
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::quiz_attempts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewQuizAttempt {
    pub player_name: String,
    pub player_email: String,
    pub topic: Option<String>,
    pub difficulty: String,
    pub num_questions: i32,
    pub score: i32,
    pub total_questions: i32,
    pub answers_json: Option<serde_json::Value>,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::app_settings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AppSetting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::quiz_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct QuizSession {
    pub id: i32,
    pub session_uuid: String,
    pub questions_json: serde_json::Value,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::quiz_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewQuizSession {
    pub session_uuid: String,
    pub questions_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub id: i32,
    pub question_text: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerRecord {
    pub question_id: i32,
    pub question_text: String,
    pub options: Vec<String>,
    pub selected_answer: String,
    pub correct_answer: String,
    pub is_correct: bool,
}

#[derive(Debug, Deserialize)]
pub struct QuizSetupForm {
    pub player_name: String,
    pub player_email: String,
    pub topic: String,
    pub difficulty: Option<String>,
    pub num_questions: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct QuizSettingsForm {
    pub api_key: String,
    pub model: String,
}
