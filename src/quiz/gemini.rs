use serde::{Deserialize, Serialize};
use crate::utils::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiQuestion {
    pub question_text: String,
    pub options: Vec<String>,
    pub correct_answer: String,
}

pub async fn generate_questions(
    api_key: &str,
    model: &str,
    topic: &str,
    difficulty: &str,
    num_questions: i32,
) -> Result<Vec<GeminiQuestion>, AppError> {
    let prompt = format!(
        r#"You are a quiz generator. Generate exactly {} multiple-choice questions about "{}" at {} level.
Each question must have exactly 4 options with one correct answer.
Return ONLY a valid JSON array (no markdown, no explanations, no code fences) with this exact structure:
[
  {{
    "question_text": "Question here",
    "options": ["Option A", "Option B", "Option C", "Option D"],
    "correct_answer": "Correct option text"
  }}
]"#,
        num_questions, topic, difficulty
    );

    let gemini = gemini_rust::Gemini::with_model(api_key, model.to_string())
        .map_err(|e| AppError::Internal(format!("Failed to create Gemini client: {}", e)))?;

    let response = gemini
        .generate_content()
        .with_user_message(prompt.clone())
        .execute()
        .await
        .map_err(|e| AppError::Internal(format!("Gemini API error: {}", e)))?;

    let text = response.text();

    let cleaned = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let questions: Vec<GeminiQuestion> = serde_json::from_str(cleaned).map_err(|e| {
        AppError::Internal(format!(
            "Failed to parse Gemini questions from JSON: {}. Raw: {}",
            e,
            &text[..text.len().min(300)]
        ))
    })?;

    if questions.is_empty() {
        return Err(AppError::Internal("Gemini generated zero questions".to_string()));
    }

    Ok(questions)
}
