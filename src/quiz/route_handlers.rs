use askama::Template;
use axum::{
    Extension, Form, Router,
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use uuid::Uuid;

use super::gemini::{self, GeminiQuestion};
use super::models::{AnswerRecord, NewQuizAttempt, NewQuizSession, QuizAttemptDb, QuizQuestion, QuizSetupForm, QuizSettingsForm};
use super::quiz_repository::QuizRepository;
use crate::middlewares::{session_middleware, user_session_middleware};
use crate::models::CustomSession;
use crate::state::AppState;
use crate::utils::error::AppError;

pub async fn quiz_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/play-quiz",
            get(play_quiz_page).post(generate_quiz_handler)
                .layer(axum::middleware::from_fn_with_state(state.clone(), user_session_middleware)))
        .route("/play-quiz/submit", post(submit_quiz_handler)
            .layer(axum::middleware::from_fn_with_state(state.clone(), user_session_middleware)))
        .route("/admin/quiz/list", get(admin_quiz_list_page)
            .layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)))
        .route("/admin/quiz/{attempt_id}/detail", get(admin_quiz_detail_page)
            .layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)))
        .route("/admin/quiz/settings", get(admin_quiz_settings_page).post(admin_quiz_settings_handler)
            .layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)))
}

#[derive(Template)]
#[template(path = "play_quiz.html")]
struct PlayQuizTemplate {
    api_configured: bool,
    player_name: String, player_email: String,
    selected_topic: String, selected_difficulty: String,
    selected_num_questions: i32,
    questions: Vec<QuizQuestion>,
    session_uuid: String,
    flash: Option<crate::models::FlashData>,
}

pub async fn play_quiz_page(
    State(state): State<AppState>,
    Extension(session): Extension<CustomSession>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let user = get_oauth_user_by_email(&mut conn, &session.user_id).await?;
    let flash = crate::session_backend::take_flash(&mut conn, &session).await.1;
    let repo = QuizRepository::new(&mut conn);
    let api_key = repo.get_setting("gemini_api_key").await;

    Ok(Html(PlayQuizTemplate {
        api_configured: api_key.is_ok() && api_key.unwrap().is_some(),
        player_name: user.name, player_email: user.email,
        selected_topic: String::new(), selected_difficulty: String::new(),
        selected_num_questions: 10,
        questions: Vec::new(), session_uuid: String::new(),
        flash: Some(flash),
    }.render()?))
}

pub async fn generate_quiz_handler(
    State(state): State<AppState>,
    Extension(session): Extension<CustomSession>,
    Form(form): Form<QuizSetupForm>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let mut flash = crate::session_backend::take_flash(&mut conn, &session).await.1;
    let user = get_oauth_user_by_email(&mut conn, &session.user_id).await?;

    let api_key = QuizRepository::new(&mut conn).get_setting("gemini_api_key").await;
    let model = QuizRepository::new(&mut conn).get_setting("gemini_model").await;

    let difficulty = form.difficulty.clone().unwrap_or_else(|| "beginner".to_string()).to_lowercase();
    let num_q = form.num_questions.unwrap_or(10).max(1).min(50);

    if api_key.is_err() || api_key.as_ref().unwrap().is_none() {
        flash.error = Some("Quiz API not configured.".to_string());
        return Ok(Html(PlayQuizTemplate {
            api_configured: false,
            player_name: user.name, player_email: user.email,
            selected_topic: String::new(), selected_difficulty: String::new(),
            selected_num_questions: 10,
            questions: Vec::new(), session_uuid: String::new(),
            flash: Some(flash),
        }.render()?).into_response());
    }

    match gemini::generate_questions(
        &api_key.unwrap().unwrap(), &model.unwrap_or(Some("models/gemini-2.5-flash".to_string())).unwrap(),
        &form.topic, &difficulty, num_q,
    ).await {
        Ok(gemini_questions) => {
            let questions_json = serde_json::to_value(&gemini_questions)
                .map_err(|e| AppError::Internal(format!("JSON: {}", e)))?;
            let session_uuid_val = Uuid::new_v4().to_string();
            QuizRepository::new(&mut conn).insert_quiz_session(&NewQuizSession {
                session_uuid: session_uuid_val.clone(), questions_json,
            }).await?;

            let questions: Vec<QuizQuestion> = gemini_questions.into_iter().enumerate()
                .map(|(i, q)| QuizQuestion { id: (i+1) as i32, question_text: q.question_text, options: q.options })
                .collect();

            Ok(Html(PlayQuizTemplate {
                api_configured: true,
                player_name: user.name, player_email: user.email,
                selected_topic: form.topic, selected_difficulty: difficulty,
                selected_num_questions: num_q, questions,
                session_uuid: session_uuid_val, flash: None,
            }.render()?).into_response())
        }
        Err(_) => {
            flash.error = Some("Failed to generate quiz. Try a different topic.".to_string());
            Ok(Html(PlayQuizTemplate {
                api_configured: true,
                player_name: user.name, player_email: user.email,
                selected_topic: form.topic, selected_difficulty: difficulty,
                selected_num_questions: num_q,
                questions: Vec::new(), session_uuid: String::new(),
                flash: Some(flash),
            }.render()?).into_response())
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct QuizSubmission {
    player_name: String, player_email: String, topic: String,
    difficulty: String, num_questions: i32, session_uuid: String,
    #[serde(flatten)]
    answers: std::collections::HashMap<String, String>,
}

#[derive(Template)]
#[template(path = "quiz_result.html")]
struct QuizResultTemplate {
    player_name: String, player_email: String,
    score: i32, total: i32, percentage: String,
    display_score_class: String, display_alert_class: String,
    display_message: String, answers: Vec<AnswerRecord>,
    flash: Option<crate::models::FlashData>,
}

pub async fn submit_quiz_handler(
    State(state): State<AppState>,
    Extension(session): Extension<CustomSession>,
    Form(form): Form<QuizSubmission>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let _flash = crate::session_backend::take_flash(&mut conn, &session).await.1;
    if form.session_uuid.trim().is_empty() {
        return Ok(Redirect::to("/play-quiz").into_response());
    }

    let quiz_session = QuizRepository::new(&mut conn).find_quiz_session_by_uuid(&form.session_uuid).await?
        .ok_or_else(|| AppError::NotFound("Session expired".into()))?;

    let session_questions: Vec<GeminiQuestion> = serde_json::from_value(quiz_session.questions_json)
        .map_err(|e| AppError::Internal(format!("Parse: {}", e)))?;

    let mut score = 0;
    let total = session_questions.len() as i32;
    let mut answer_records: Vec<AnswerRecord> = Vec::new();

    for (i, q) in session_questions.iter().enumerate() {
        let key = format!("q_{}", i+1);
        let user_ans = form.answers.get(&key).cloned().unwrap_or_default();
        let correct = user_ans.trim().to_lowercase() == q.correct_answer.trim().to_lowercase();
        if correct { score += 1; }
        answer_records.push(AnswerRecord {
            question_id: (i+1) as i32, question_text: q.question_text.clone(),
            options: q.options.clone(), selected_answer: user_ans,
            correct_answer: q.correct_answer.clone(), is_correct: correct,
        });
    }

    let pct = if total > 0 { ((score as f64 / total as f64) * 100.0).round() as i32 } else { 0 };
    let user = get_oauth_user_by_email(&mut conn, &session.user_id).await?;

    QuizRepository::new(&mut conn).insert_attempt(&NewQuizAttempt {
        player_name: user.name, player_email: user.email,
        topic: Some(form.topic), difficulty: form.difficulty,
        num_questions: form.num_questions, score, total_questions: total,
        answers_json: Some(serde_json::to_value(&answer_records).unwrap()),
        oauth_user_id: Some(user.id),
    }).await?;
    QuizRepository::new(&mut conn).delete_quiz_session(&form.session_uuid).await?;

    let (cls, alert, msg) = match pct {
        90..=100 => ("display-score-excellent", "alert alert-success", "Excellent!"),
        70..=89 => ("display-score-good", "alert alert-info", "Great job!"),
        50..=69 => ("display-score-good", "alert alert-warning", "Good effort!"),
        _ => ("display-score-poor", "alert alert-danger", "Keep practicing!"),
    };

    Ok(Html(QuizResultTemplate {
        player_name: form.player_name, player_email: form.player_email,
        score, total, percentage: pct.to_string(),
        display_score_class: cls.to_string(), display_alert_class: alert.to_string(),
        display_message: msg.to_string(), answers: answer_records, flash: None,
    }.render()?).into_response())
}

// ── Admin ──

fn score_color(pct: i32) -> &'static str {
    if pct >= 70 { "text-success" } else if pct >= 50 { "text-warning" } else { "text-danger" }
}

fn difficulty_badge(d: &str) -> &'static str {
    match d { "beginner" => "badge-success", "intermediate" => "badge-info", _ => "badge-warning" }
}

pub async fn admin_quiz_list_page(
    State(state): State<AppState>,
    Extension(session): Extension<CustomSession>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let flash = crate::session_backend::take_flash(&mut conn, &session).await.1;

    let all_attempts: Vec<QuizAttemptDb> = crate::schema::quiz_attempts::dsl::quiz_attempts
        .order(crate::schema::quiz_attempts::dsl::played_at.desc())
        .load(&mut conn).await
        .map_err(|e| AppError::DatabaseError(e))?;

    let total = all_attempts.len() as i64;

    Ok(Html(template::AdminQuizListTemplate {
        attempts: all_attempts,
        total,
        flash: Some(flash),
        active_nav: "quiz".to_string(),
    }.render()?))
}

mod template {
    use askama::Template;
    use crate::quiz::models::QuizAttemptDb;
    use crate::models::FlashData;

    #[derive(Template)]
    #[template(path = "admin/quiz_list.html")]
    pub struct AdminQuizListTemplate {
        pub attempts: Vec<QuizAttemptDb>,
        pub total: i64,
        pub flash: Option<FlashData>,
        pub active_nav: String,
    }
}

pub async fn admin_quiz_detail_page(
    State(state): State<AppState>,
    Extension(session): Extension<CustomSession>,
    Path(attempt_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let flash = crate::session_backend::take_flash(&mut conn, &session).await.1;

    let attempt: QuizAttemptDb = crate::schema::quiz_attempts::dsl::quiz_attempts
        .filter(crate::schema::quiz_attempts::dsl::id.eq(attempt_id))
        .first(&mut conn).await
        .map_err(|_| AppError::NotFound("Quiz attempt not found".into()))?;

    let answers: Vec<AnswerRecord> = attempt.answers_json
        .as_ref().and_then(|j| serde_json::from_value(j.clone()).ok()).unwrap_or_default();
    let pct = if attempt.total_questions > 0 {
        ((attempt.score as f64 / attempt.total_questions as f64) * 100.0).round() as i32
    } else { 0 };

    Ok(Html(AdminQuizDetailTemplate {
        player_name: attempt.player_name, player_email: attempt.player_email,
        topic: attempt.topic.unwrap_or_default(), difficulty: attempt.difficulty,
        score: attempt.score, total: attempt.total_questions,
        percentage: pct.to_string(), score_color: score_color(pct).to_string(),
        played_at: attempt.played_at.format("%Y-%m-%d %H:%M").to_string(),
        answers, back_url: "/admin/quiz/list".to_string(),
        flash: Some(flash), active_nav: "quiz".to_string(),
    }.render()?))
}

#[derive(Template)]
#[template(path = "admin/quiz_detail.html")]
struct AdminQuizDetailTemplate {
    player_name: String, player_email: String,
    topic: String, difficulty: String, score: i32, total: i32,
    percentage: String, score_color: String, played_at: String,
    answers: Vec<AnswerRecord>, back_url: String,
    flash: Option<crate::models::FlashData>,
    active_nav: String,
}

pub async fn admin_quiz_settings_page(
    State(state): State<AppState>,
    Extension(session): Extension<CustomSession>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let flash = crate::session_backend::take_flash(&mut conn, &session).await.1;
    let repo = QuizRepository::new(&mut conn);
    let api_key_str = repo.get_setting("gemini_api_key").await.unwrap_or(None).unwrap_or_default();
    let model_str = QuizRepository::new(&mut conn).get_setting("gemini_model").await.unwrap_or(None).unwrap_or_else(|| "models/gemini-2.5-flash".to_string());

    #[derive(Template)]
    #[template(path = "admin/quiz_settings.html")]
    struct T {
        api_key: String, model: String,
        api_key_set: bool, selected_model: String,
        flash: Option<crate::models::FlashData>, active_nav: String,
    }

    Ok(Html(T {
        api_key: api_key_str.clone(), model: model_str.clone(),
        api_key_set: !api_key_str.is_empty(), selected_model: model_str,
        flash: Some(flash), active_nav: "quiz-settings".to_string(),
    }.render()?))
}

pub async fn admin_quiz_settings_handler(
    State(state): State<AppState>,
    Extension(session): Extension<CustomSession>,
    Form(form): Form<QuizSettingsForm>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let repo = QuizRepository::new(&mut conn);
    QuizRepository::new(&mut conn).upsert_setting("gemini_api_key", &form.api_key).await?;
    QuizRepository::new(&mut conn).upsert_setting("gemini_model", &form.model).await?;
    crate::session_backend::set_flash(&mut conn, &session, Some("Quiz settings updated.".to_string()), None).await?;
    Ok(Redirect::to("/admin/quiz/settings"))
}

async fn get_oauth_user_by_email(
    conn: &mut crate::db::PooledConn,
    user_email: &str,
) -> Result<crate::oauth::models::OAuthUser, AppError> {
    crate::schema::oauth_users::dsl::oauth_users
        .filter(crate::schema::oauth_users::dsl::email.eq(user_email))
        .first::<crate::oauth::models::OAuthUser>(conn)
        .await
        .map_err(|_| AppError::NotFound("User not found".into()))
}
