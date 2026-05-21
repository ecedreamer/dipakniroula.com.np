use askama::Template;
use axum::{
    Extension, Form, Router,
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use serde::Deserialize;
use uuid::Uuid;

use super::gemini;
use super::models::{AnswerRecord, NewQuizAttempt, NewQuizSession, QuizQuestion, QuizSetupForm, QuizSettingsForm};
use super::quiz_repository::QuizRepository;
use crate::middlewares::session_middleware;
use crate::state::AppState;
use crate::utils::error::AppError;

pub async fn quiz_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/play-quiz", get(play_quiz_page).post(generate_quiz_handler))
        .route("/play-quiz/submit", post(submit_quiz_handler))
        .route(
            "/admin/quiz/list",
            get(admin_quiz_list_page)
                .layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
        .route(
            "/admin/quiz/{attempt_id}/detail",
            get(admin_quiz_detail_page)
                .layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
        .route(
            "/admin/quiz/settings",
            get(admin_quiz_settings_page)
                .post(admin_quiz_settings_handler)
                .layer(axum::middleware::from_fn_with_state(state.clone(), session_middleware)),
        )
}

#[derive(Template)]
#[template(path = "play_quiz.html")]
struct PlayQuizTemplate {
    api_configured: bool,
    player_name: String,
    player_email: String,
    selected_topic: String,
    selected_difficulty: String,
    selected_num_questions: i32,
    questions: Vec<QuizQuestion>,
    session_uuid: String,
    flash: Option<crate::models::FlashData>,
}

pub async fn play_quiz_page(
    State(state): State<AppState>,
    session: Option<Extension<crate::models::CustomSession>>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let flash = if let Some(Extension(s)) = session {
        crate::session_backend::take_flash(&mut conn, &s).await.1
    } else {
        crate::models::FlashData::default()
    };

    let api_key = QuizRepository::new(&mut conn)
        .get_setting("gemini_api_key")
        .await?;

    let context = PlayQuizTemplate {
        api_configured: api_key.is_some(),
        player_name: String::new(),
        player_email: String::new(),
        selected_topic: String::new(),
        selected_difficulty: "beginner".to_string(),
        selected_num_questions: 10,
        questions: Vec::new(),
        session_uuid: String::new(),
        flash: Some(flash),
    };

    Ok(Html(context.render()?).into_response())
}

pub async fn generate_quiz_handler(
    State(state): State<AppState>,
    session: Option<Extension<crate::models::CustomSession>>,
    Form(form): Form<QuizSetupForm>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let flash = if let Some(Extension(s)) = session {
        crate::session_backend::take_flash(&mut conn, &s).await.1
    } else {
        crate::models::FlashData::default()
    };

    if form.player_name.trim().is_empty()
        || form.player_email.trim().is_empty()
        || form.topic.trim().is_empty()
    {
        return Ok(Redirect::to("/play-quiz").into_response());
    }

    let encrypted_key = QuizRepository::new(&mut conn)
        .get_setting("gemini_api_key")
        .await?
        .ok_or_else(|| AppError::Internal("Gemini API key not configured".to_string()))?;
    let api_key = crate::utils::crypto::decrypt(&encrypted_key)
        .map_err(|e| AppError::Internal(format!("Failed to decrypt API key: {}", e)))?;
    let model = QuizRepository::new(&mut conn)
        .get_setting("gemini_model")
        .await?
        .unwrap_or_else(|| "models/gemini-2.5-flash".to_string());

    let difficulty = form
        .difficulty
        .clone()
        .unwrap_or_else(|| "beginner".to_string());
    let num_q = form.num_questions.unwrap_or(10).max(1).min(50);

    let gemini_questions = match gemini::generate_questions(
        &api_key,
        &model,
        form.topic.trim(),
        &difficulty,
        num_q,
    )
    .await
    {
        Ok(q) => q,
        Err(_e) => {
            let error_msg = format!("Failed to generate quiz. Please try again or select a different topic/model.");
            let context = PlayQuizTemplate {
                api_configured: true,
                player_name: form.player_name,
                player_email: form.player_email,
                selected_topic: form.topic,
                selected_difficulty: difficulty,
                selected_num_questions: num_q,
                questions: Vec::new(),
                session_uuid: String::new(),
                flash: Some(crate::models::FlashData {
                    success: None,
                    error: Some(error_msg),
                }),
            };
            return Ok(Html(context.render()?).into_response());
        }
    };

    let questions_json = serde_json::to_value(&gemini_questions)
        .map_err(|e| AppError::Internal(format!("JSON serialization error: {}", e)))?;

    let session_uuid = Uuid::new_v4().to_string();

    QuizRepository::new(&mut conn)
        .insert_quiz_session(&NewQuizSession {
            session_uuid: session_uuid.clone(),
            questions_json,
        })
        .await?;

    let questions: Vec<QuizQuestion> = gemini_questions
        .into_iter()
        .enumerate()
        .map(|(i, q)| QuizQuestion {
            id: (i + 1) as i32,
            question_text: q.question_text,
            options: q.options,
        })
        .collect();

    let context = PlayQuizTemplate {
        api_configured: true,
        player_name: form.player_name,
        player_email: form.player_email,
        selected_topic: form.topic,
        selected_difficulty: difficulty,
        selected_num_questions: num_q,
        questions,
        session_uuid,
        flash: Some(flash),
    };

    Ok(Html(context.render()?).into_response())
}

#[derive(Debug, Deserialize)]
pub struct QuizSubmission {
    player_name: String,
    player_email: String,
    topic: String,
    difficulty: String,
    num_questions: i32,
    session_uuid: String,
    #[serde(flatten)]
    answers: std::collections::HashMap<String, String>,
}

#[derive(Template)]
#[template(path = "quiz_result.html")]
struct QuizResultTemplate {
    player_name: String,
    player_email: String,
    score: i32,
    total: i32,
    percentage: String,
    display_score_class: String,
    display_alert_class: String,
    display_message: String,
    answers: Vec<AnswerRecord>,
    flash: Option<crate::models::FlashData>,
}

pub async fn submit_quiz_handler(
    State(state): State<AppState>,
    session: Option<Extension<crate::models::CustomSession>>,
    Form(form): Form<QuizSubmission>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let flash = if let Some(Extension(s)) = session {
        crate::session_backend::take_flash(&mut conn, &s).await.1
    } else {
        crate::models::FlashData::default()
    };

    if form.session_uuid.trim().is_empty() {
        return Ok(Redirect::to("/play-quiz").into_response());
    }

    let session_data = QuizRepository::new(&mut conn)
        .find_quiz_session_by_uuid(&form.session_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound("Quiz session".to_string()))?;

    let gemini_questions: Vec<gemini::GeminiQuestion> = serde_json::from_value(
        session_data.questions_json,
    )
    .map_err(|e| AppError::Internal(format!("Failed to parse session questions: {}", e)))?;

    let _ = QuizRepository::new(&mut conn)
        .delete_quiz_session(&form.session_uuid)
        .await;

    let mut score: i32 = 0;
    let mut answer_records: Vec<AnswerRecord> = Vec::new();

    for (i, q) in gemini_questions.iter().enumerate() {
        let key = format!("q_{}", i + 1);
        let selected = form.answers.get(&key).cloned().unwrap_or_default();
        let is_correct = selected.trim() == q.correct_answer.trim();
        if is_correct {
            score += 1;
        }

        answer_records.push(AnswerRecord {
            question_id: (i + 1) as i32,
            question_text: q.question_text.clone(),
            options: q.options.clone(),
            selected_answer: selected,
            correct_answer: q.correct_answer.clone(),
            is_correct,
        });
    }

    let total = gemini_questions.len() as i32;
    let pct = if total > 0 {
        (score as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    let percentage = format!("{:.0}", pct);
    let display_score_class = if pct >= 80.0 {
        "text-success"
    } else if pct >= 50.0 {
        "text-warning"
    } else {
        "text-danger"
    };
    let display_message = if pct >= 80.0 {
        "Excellent work! You have great knowledge!"
    } else if pct >= 50.0 {
        "Good effort! Keep learning to improve."
    } else {
        "Keep studying! You'll do better next time."
    };
    let display_alert_class = if pct >= 80.0 {
        "alert-success"
    } else if pct >= 50.0 {
        "alert-warning"
    } else {
        "alert-danger"
    };

    let new_attempt = NewQuizAttempt {
        player_name: form.player_name.clone(),
        player_email: form.player_email.clone(),
        topic: Some(form.topic.clone()),
        difficulty: form.difficulty.clone(),
        num_questions: form.num_questions,
        score,
        total_questions: total,
        answers_json: Some(serde_json::to_value(&answer_records).unwrap_or_default()),
    };

    let _ = QuizRepository::new(&mut conn)
        .insert_attempt(&new_attempt)
        .await;

    let context = QuizResultTemplate {
        player_name: form.player_name,
        player_email: form.player_email,
        score,
        total,
        percentage,
        display_score_class: display_score_class.to_string(),
        display_alert_class: display_alert_class.to_string(),
        display_message: display_message.to_string(),
        answers: answer_records,
        flash: Some(flash),
    };

    Ok(Html(context.render()?).into_response())
}

#[derive(Deserialize)]
pub struct QuizPagination {
    pub page: Option<i64>,
}

#[allow(dead_code)]
struct QuizAttemptRow {
    id: Option<i32>,
    player_name: String,
    player_email: String,
    display_topic: String,
    difficulty: String,
    difficulty_badge_class: String,
    score_badge_class: String,
    score_display: String,
    played_at_display: String,
}

#[derive(Template)]
#[template(path = "admin/quiz_list.html")]
struct AdminQuizListTemplate {
    attempts: Vec<QuizAttemptRow>,
    active_nav: String,
    current_page: i64,
    total_pages: i64,
    pages: Vec<i64>,
    total_count: i64,
    flash: Option<crate::models::FlashData>,
}

pub async fn admin_quiz_list_page(
    State(state): State<AppState>,
    Extension(session): Extension<crate::models::CustomSession>,
    Query(pagination): Query<QuizPagination>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let flash = crate::session_backend::take_flash(&mut conn, &session).await.1;

    let page = pagination.page.unwrap_or(1).max(1);
    let per_page: i64 = 15;

    let repo = QuizRepository::new(&mut conn);
    let (results, total) = repo.find_all(page, per_page).await?;

    let attempts: Vec<QuizAttemptRow> = results
        .into_iter()
        .map(|a| {
            let pct = if a.total_questions > 0 {
                a.score as f64 / a.total_questions as f64
            } else {
                0.0
            };
            let score_badge_class = if pct >= 0.8 {
                "text-success"
            } else if pct >= 0.5 {
                "text-warning"
            } else {
                "text-danger"
            };
            let difficulty_badge_class = match a.difficulty.as_str() {
                "beginner" => "badge-success",
                "intermediate" => "badge-info",
                _ => "badge-warning",
            };
            QuizAttemptRow {
                id: a.id,
                player_name: a.player_name,
                player_email: a.player_email,
                display_topic: a.topic.unwrap_or_else(|| "All".to_string()),
                difficulty: a.difficulty,
                difficulty_badge_class: difficulty_badge_class.to_string(),
                score_badge_class: score_badge_class.to_string(),
                score_display: format!("{}/{}", a.score, a.total_questions),
                played_at_display: a.played_at.format("%Y-%m-%d %H:%M").to_string(),
            }
        })
        .collect();

    let total_pages = if total == 0 {
        1
    } else {
        (total + per_page - 1) / per_page
    };
    let pages_vec: Vec<i64> = (1..=total_pages).collect();

    let context = AdminQuizListTemplate {
        attempts,
        active_nav: "quiz".to_string(),
        current_page: page,
        total_pages,
        pages: pages_vec,
        total_count: total,
        flash: Some(flash),
    };

    Ok(Html(context.render()?).into_response())
}

#[derive(Template)]
#[template(path = "admin/quiz_detail.html")]
#[allow(dead_code)]
struct AdminQuizDetailTemplate {
    player_name: String,
    player_email: String,
    display_topic: String,
    difficulty: String,
    difficulty_badge_class: String,
    score_display: String,
    percentage_display: String,
    score_badge_class: String,
    played_at_display: String,
    answers: Vec<AnswerRecord>,
    active_nav: String,
    flash: Option<crate::models::FlashData>,
}

pub async fn admin_quiz_detail_page(
    State(state): State<AppState>,
    Extension(session): Extension<crate::models::CustomSession>,
    Path(attempt_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let flash = crate::session_backend::take_flash(&mut conn, &session).await.1;

    let repo = QuizRepository::new(&mut conn);
    let attempt = repo.find_by_id(attempt_id).await?;

    let answers: Vec<AnswerRecord> = attempt
        .answers_json
        .clone()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    let pct = if attempt.total_questions > 0 {
        attempt.score as f64 / attempt.total_questions as f64 * 100.0
    } else {
        0.0
    };
    let score_badge_class = if pct >= 80.0 {
        "text-success"
    } else if pct >= 50.0 {
        "text-warning"
    } else {
        "text-danger"
    };
    let difficulty_badge_class = match attempt.difficulty.as_str() {
        "beginner" => "badge-success",
        "intermediate" => "badge-info",
        _ => "badge-warning",
    };

    let context = AdminQuizDetailTemplate {
        player_name: attempt.player_name,
        player_email: attempt.player_email,
        display_topic: attempt.topic.unwrap_or_else(|| "All".to_string()),
        difficulty: attempt.difficulty,
        difficulty_badge_class: difficulty_badge_class.to_string(),
        score_display: format!("{} / {}", attempt.score, attempt.total_questions),
        percentage_display: format!("{:.1}%", pct),
        score_badge_class: score_badge_class.to_string(),
        played_at_display: attempt.played_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        answers,
        active_nav: "quiz".to_string(),
        flash: Some(flash),
    };

    Ok(Html(context.render()?).into_response())
}

#[derive(Template)]
#[template(path = "admin/quiz_settings.html")]
struct AdminQuizSettingsTemplate {
    api_key_set: bool,
    selected_model: String,
    active_nav: String,
    flash: Option<crate::models::FlashData>,
}

pub async fn admin_quiz_settings_page(
    State(state): State<AppState>,
    Extension(session): Extension<crate::models::CustomSession>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let flash = crate::session_backend::take_flash(&mut conn, &session).await.1;

    let api_key = QuizRepository::new(&mut conn)
        .get_setting("gemini_api_key")
        .await?;
    let model = QuizRepository::new(&mut conn)
        .get_setting("gemini_model")
        .await?
        .unwrap_or_else(|| "models/gemini-2.5-flash".to_string());

    let context = AdminQuizSettingsTemplate {
        api_key_set: api_key.is_some(),
        selected_model: model,
        active_nav: "quiz-settings".to_string(),
        flash: Some(flash),
    };

    Ok(Html(context.render()?).into_response())
}

pub async fn admin_quiz_settings_handler(
    State(state): State<AppState>,
    Extension(session): Extension<crate::models::CustomSession>,
    Form(form): Form<QuizSettingsForm>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;

    let encrypted = crate::utils::crypto::encrypt(form.api_key.trim())
        .map_err(|e| AppError::Internal(format!("Failed to encrypt API key: {}", e)))?;
    QuizRepository::new(&mut conn)
        .upsert_setting("gemini_api_key", &encrypted)
        .await?;
    QuizRepository::new(&mut conn)
        .upsert_setting("gemini_model", form.model.trim())
        .await?;

    crate::session_backend::set_flash(
        &mut conn,
        &session,
        Some("Quiz settings saved successfully.".to_string()),
        None,
    )
    .await?;

    Ok(Redirect::to("/admin/quiz/settings"))
}
