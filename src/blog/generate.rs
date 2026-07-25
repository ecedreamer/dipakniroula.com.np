use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::AsyncWriteExt;
use crate::utils::error::AppError;

#[derive(Debug, Deserialize, Default)]
pub struct GeneratedBlog {
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub image_prompt: Option<String>,
}

fn clean_json_text(raw: &str) -> String {
    let cleaned = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    cleaned.to_string()
}

fn try_parse_json(text: &str) -> Result<GeneratedBlog, String> {
    serde_json::from_str::<GeneratedBlog>(text)
        .map_err(|e| format!("{}", e))
}

fn extract_string_value(text: &str, key: &str) -> Option<String> {
    let search = &format!("\"{}\":\"", key);
    let start = text.find(search)?;
    let value_start = start + search.len();
    let mut result = String::new();
    for c in text[value_start..].chars() {
        if c == '\\' {
            continue;
        }
        if c == '"' {
            return Some(result);
        }
        result.push(c);
    }
    // Reached end of text without closing quote - return what we have
    Some(result)
}

fn try_parse_with_fixes(text: &str) -> Result<GeneratedBlog, String> {
    // Try as-is
    if let Ok(blog) = try_parse_json(text) {
        return Ok(blog);
    }
    // Try appending closing brace (in case of truncation)
    let trimmed = text.trim_end();
    if !trimmed.ends_with('}') {
        let with_brace = format!("{}}}", trimmed);
        if let Ok(blog) = try_parse_json(&with_brace) {
            return Ok(blog);
        }
        // Try appending closing quote + brace (content string was truncated)
        let with_quote_brace = format!("{}}}\"}}", trimmed);
        if let Ok(blog) = try_parse_json(&with_quote_brace) {
            return Ok(blog);
        }
    }
    // Try fixing newlines inside strings
    let mut fixed = String::with_capacity(text.len());
    let mut in_string = false;
    let mut escaped = false;

    for c in text.chars() {
        if escaped {
            fixed.push(c);
            escaped = false;
            continue;
        }
        if c == '\\' {
            fixed.push(c);
            escaped = true;
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            fixed.push(c);
            continue;
        }
        if in_string && (c == '\n' || c == '\r') {
            fixed.push_str("\\n");
            continue;
        }
        fixed.push(c);
    }

    // Try with fixed newlines + various closings
    for suffix in &["}", "\"}", "\"", "null\"}"] {
        let attempt = format!("{}{}", fixed.trim_end(), suffix);
        if let Ok(blog) = try_parse_json(&attempt) {
            return Ok(blog);
        }
    }

    // Last resort: manually extract title and content
    let title = extract_string_value(text, "title").unwrap_or_default();
    let content = extract_string_value(text, "content").unwrap_or_default();
    if !title.is_empty() && !content.is_empty() {
        return Ok(GeneratedBlog { title, content, image_prompt: None });
    }

    try_parse_json(&fixed)
}

pub async fn generate_blog(
    api_key: &str,
    model: &str,
    topic: &str,
    outline: &str,
    category: &str,
) -> Result<GeneratedBlog, AppError> {
    let prompt = format!(
        r#"You are a professional tech blogger. Write a blog post about "{}".

Topic context / outline:
{}

Category: {}

CRITICAL REQUIREMENTS - Follow exactly:
1. Write in a professional but engaging tone
2. Use proper HTML formatting (h2 for sections, p for paragraphs, ul/ol for lists, pre/code for code snippets, blockquote for quotes)
3. Include a compelling title
4. Structure with clear sections and subsections
5. Make it thorough and informative (800-1500 words)
6. Also generate a short image_prompt (max 100 chars) describing what a featured image for this blog should look like
7. Return ONLY a valid JSON object - NO markdown, NO code fences, NO explanations before or after
8. The ENTIRE response must be ONLY the JSON object, nothing else
9. The "content" field must be a single line of HTML with NO literal newlines
10. Use \n (escaped backslash-n) instead of actual newlines in the content field
11. The response must start with {{ and end with }}

Use this exact structure:
{{"title":"Your Compelling Blog Title Here","content":"<h2>Section 1</h2><p>Content here...</p><h2>Section 2</h2><p>More content...</p>","image_prompt":"A modern illustration of..."}}"#,
        topic, outline, category
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
    let cleaned = clean_json_text(&text);

    let blog = try_parse_with_fixes(&cleaned)
        .map_err(|e| {
            AppError::Internal(format!(
                "Failed to parse generated blog: {}. Raw: {}",
                e,
                &text[..text.len().min(300)]
            ))
        })?;

    if blog.title.is_empty() || blog.content.is_empty() {
        return Err(AppError::Internal("Gemini generated an incomplete blog".to_string()));
    }

    Ok(blog)
}

fn sanitize_filename(title: &str) -> String {
    title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == ' ' { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase()
}

fn fallback_svg_image(title: &str) -> Vec<u8> {
    let colors = [
        ("1e40af", "3b82f6", "60a5fa"),
        ("0d9488", "14b8a6", "5eead4"),
        ("6d28d9", "8b5cf6", "c4b5fd"),
        ("be123c", "ef4444", "fca5a5"),
    ];
    let idx = title.len() % colors.len();
    let (c1, c2, c3) = colors[idx];
    let safe = title
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");

    let truncated: String = safe.chars().take(40).collect();

    let mut s = String::from(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1200\" height=\"630\" viewBox=\"0 0 1200 630\">"
    );
    s.push_str("<defs>");
    s.push_str(&format!("<linearGradient id=\"bg\" x1=\"0%\" y1=\"0%\" x2=\"100%\" y2=\"100%\"><stop offset=\"0%\" stop-color=\"{}\"/><stop offset=\"50%\" stop-color=\"{}\"/><stop offset=\"100%\" stop-color=\"{}\"/></linearGradient>",
        format!("#{}", c1), format!("#{}", c2), format!("#{}", c3)));
    s.push_str(&format!("<linearGradient id=\"g1\" x1=\"0%\" y1=\"100%\" x2=\"100%\" y2=\"0%\"><stop offset=\"0%\" stop-color=\"#ffffff\" stop-opacity=\"0.08\"/><stop offset=\"100%\" stop-color=\"#ffffff\" stop-opacity=\"0\"/></linearGradient>"));
    s.push_str(&format!("<linearGradient id=\"g2\" x1=\"100%\" y1=\"100%\" x2=\"0%\" y2=\"0%\"><stop offset=\"0%\" stop-color=\"#ffffff\" stop-opacity=\"0.05\"/><stop offset=\"100%\" stop-color=\"#ffffff\" stop-opacity=\"0\"/></linearGradient>"));
    s.push_str("<filter id=\"glow\"><feGaussianBlur stdDeviation=\"3\" result=\"blur\"/><feMerge><feMergeNode in=\"blur\"/><feMergeNode in=\"SourceGraphic\"/></feMerge></filter>");
    s.push_str("</defs>");
    s.push_str("<rect width=\"1200\" height=\"630\" fill=\"url(#bg)\"/>");
    s.push_str("<rect width=\"1200\" height=\"630\" fill=\"url(#g1)\"/>");
    s.push_str("<rect width=\"1200\" height=\"630\" fill=\"url(#g2)\"/>");
    s.push_str("<circle cx=\"1000\" cy=\"80\" r=\"300\" fill=\"#ffffff\" fill-opacity=\"0.04\"/>");
    s.push_str("<circle cx=\"200\" cy=\"550\" r=\"250\" fill=\"#ffffff\" fill-opacity=\"0.04\"/>");
    s.push_str("<circle cx=\"650\" cy=\"315\" r=\"200\" fill=\"#ffffff\" fill-opacity=\"0.03\"/>");
    s.push_str("<rect x=\"80\" y=\"200\" width=\"6\" height=\"230\" rx=\"3\" fill=\"#ffffff\" fill-opacity=\"0.15\"/>");
    s.push_str(&format!("<text x=\"110\" y=\"270\" font-family=\"system-ui,-apple-system,sans-serif\" font-size=\"56\" font-weight=\"800\" fill=\"#ffffff\" filter=\"url(#glow)\">{}</text>", truncated));
    s.push_str("<text x=\"110\" y=\"370\" font-family=\"system-ui,-apple-system,sans-serif\" font-size=\"18\" font-weight=\"400\" fill=\"#ffffff\" fill-opacity=\"0.5\">dipakniroula.com.np</text>");
    s.push_str("</svg>");
    s.into_bytes()
}

async fn try_gemini_image(api_key: &str, model: &str, title: &str, prompt: Option<&str>) -> Result<Vec<u8>, AppError> {
    let img_model = "gemini-2.0-flash-exp";
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        img_model, api_key
    );

    let text = if let Some(p) = prompt {
        format!("Generate a blog featured image about: {}\n\nStyle: Modern, clean, tech-themed with gradient colors. No text overlay needed.\n\nAdditional context: {}", title, p)
    } else {
        format!("Generate a blog featured image about: {}\n\nStyle: Modern, clean, tech-themed with gradient colors. No text overlay needed.", title)
    };

    let body = json!({
        "contents": [{"parts": [{"text": text}]}],
        "generationConfig": {
            "responseModalities": ["Text", "Image"]
        }
    });

    let client = reqwest::Client::new();
    let resp = client.post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Image API request failed: {}", e)))?;

    let data: serde_json::Value = resp.json().await
        .map_err(|e| AppError::Internal(format!("Image API response parse failed: {}", e)))?;

    if let Some(candidates) = data["candidates"].as_array() {
        for candidate in candidates {
            if let Some(parts) = candidate["content"]["parts"].as_array() {
                for part in parts {
                    if let Some(inline_data) = part["inline_data"].as_object() {
                        if let Some(mime_type) = inline_data["mime_type"].as_str() {
                            if let Some(b64) = inline_data["data"].as_str() {
                                if mime_type.starts_with("image/") {
                                    use base64::Engine as _;
                                    let bytes = base64::engine::general_purpose::STANDARD
                                        .decode(b64)
                                        .map_err(|e| AppError::Internal(format!("Base64 decode failed: {}", e)))?;
                                    return Ok(bytes);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Err(AppError::Internal("Gemini did not return an image".into()))
}

pub async fn generate_blog_image(api_key: &str, model: &str, title: &str, image_prompt: Option<&str>) -> Result<String, AppError> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let slug = sanitize_filename(title);

    // Try Gemini image generation first
    let image_data = try_gemini_image(api_key, model, title, image_prompt).await;

    let (bytes, ext) = match image_data {
        Ok(data) => (data, "png".to_string()),
        Err(_) => (fallback_svg_image(title), "svg".to_string()),
    };

    let filename = format!("media/ai_gen_{}_{}.{}", timestamp, slug, ext);

    if let Some(parent) = std::path::Path::new(&filename).parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }

    let mut file = tokio::fs::File::create(&filename).await
        .map_err(|e| AppError::Internal(format!("Failed to create image file: {}", e)))?;
    file.write_all(&bytes).await
        .map_err(|e| AppError::Internal(format!("Failed to write image file: {}", e)))?;

    Ok(filename)
}
