use askama::Template;
use diesel_async::RunQueryDsl;

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Form,
    Json,
    response::{Html, IntoResponse, Redirect},
    Extension,
};
use tokio::{
    fs::File,
    io::AsyncWriteExt,
};


use chrono::Utc;
use axum_csrf::CsrfToken;
use serde::Deserialize;
use serde_json::json;


use crate::blog::blog_repository::blog_repo::BlogRepository;
use crate::blog::models::Blog;
use crate::state::AppState;
use crate::utils::error::AppError;


#[derive(Template, Deserialize)]
#[template(path = "home.html")]
struct HomeTemplate {
    popular_blogs: Vec<Blog>,
    flash: Option<crate::models::FlashData>,
}


pub async fn home_page(
    State(state): State<AppState>,
    session: Option<Extension<crate::models::CustomSession>>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn: crate::db::PooledConn = state.get_conn().await?;

    let flash = if let Some(Extension(s)) = session {
        crate::session_backend::take_flash(&mut conn, &s).await.1
    } else {
        crate::models::FlashData::default()
    };

    let blog_repo = BlogRepository::new(&mut conn);
    let results = blog_repo.find_active_only(None, "view_count", 3).await?;

    let context = HomeTemplate {
        popular_blogs: results,
        flash: Some(flash),
    };

    Ok(Html(context.render()?))
}


#[derive(Template)]
#[template(path = "contact.html")]
struct ContactTemplate {
    authenticity_token: String,
    flash: Option<crate::models::FlashData>,
}


pub async fn contact_page(
    State(state): State<AppState>,
    session: Option<Extension<crate::models::CustomSession>>,
    token: CsrfToken
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_conn().await?;
    let flash = if let Some(Extension(s)) = session {
        crate::session_backend::take_flash(&mut conn, &s).await.1
    } else {
        crate::models::FlashData::default()
    };

    let context = ContactTemplate {
        authenticity_token: token.authenticity_token().map_err(|e| AppError::Internal(format!("CSRF Error: {}", e)))?,
        flash: Some(flash),
    };
    Ok((token, Html(context.render()?)))
}


#[derive(Deserialize, Debug)]
pub struct ContactForm {
    authenticity_token: String,
    full_name: String,
    email: String,
    mobile: Option<String>,
    subject: String,
    message: String,
    website_url: Option<String>,
}

pub async fn contact_form_handler(
    State(state): State<AppState>,
    session: Option<Extension<crate::models::CustomSession>>,
    token: CsrfToken, 
    Form(form): Form<ContactForm>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("Contact form submission from: {}", form.full_name);
    
    // Honeypot check
    if form.website_url.is_some() && !form.website_url.as_ref().unwrap().is_empty() {
        tracing::warn!("Honeypot triggered by: {}", form.full_name);
        return Ok(Redirect::to("/contact").into_response());
    }

    if token.verify(&form.authenticity_token).is_err() {
        tracing::error!("Invalid CSRF token from: {}", form.full_name);
        return Ok(Redirect::to("/contact").into_response());
    }

    // Basic length validation
    if form.full_name.len() > 100 || form.email.len() > 100 || form.subject.len() > 200 || form.message.len() > 5000 {
        tracing::error!("Input too long from: {}", form.full_name);
        return Ok((StatusCode::BAD_REQUEST, "Input too long").into_response());
    }

    let date_time_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mobile = form.mobile.unwrap_or_default();

    use crate::schema::messages;

    let contact_message = crate::models::NewContactMessage {
        full_name: form.full_name.as_str(),
        email: form.email.as_str(),
        mobile: Some(mobile.as_str()),
        subject: form.subject.as_str(),
        message: form.message.as_str(),
        date_sent: date_time_str.as_str(),
    };
    let mut conn: crate::db::PooledConn = state.get_conn().await?;

    diesel::insert_into(messages::table)
        .values(&contact_message)
        .execute(&mut conn)
        .await?;

    // Set flash message
    let (headers, sess) = if let Some(Extension(s)) = session {
        (axum::http::HeaderMap::new(), s)
    } else {
        let s = crate::session_backend::create_session(&mut conn, "anonymous".to_string()).await?;
        let mut h = axum::http::HeaderMap::new();
        h.insert(
            axum::http::header::SET_COOKIE,
            axum::http::HeaderValue::from_str(&format!("session_id={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=3600", s.session_id)).unwrap()
        );
        (h, s)
    };

    crate::session_backend::set_flash(&mut conn, &sess, Some("Message sent successfully! I will get back to you soon.".to_string()), None).await?;

    Ok((headers, Redirect::to("/")).into_response())
}


pub async fn summernote_upload(
    State(_state): State<AppState>,
    mut multipart: Multipart
) -> Result<Json<serde_json::Value>, AppError> {
    let mut image_path = String::new();
    while let Ok(Some(mut field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();
        tracing::info!("{}", field_name);

        if field_name == "file" {
            let file_name = field.file_name().unwrap_or("").to_string();

            if !file_name.is_empty() {
                let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
                image_path = format!("{}{}_{}", "media/summernote/", timestamp, file_name);
                let mut file = File::create(image_path.clone()).await?;
                while let Ok(Some(chunk)) = field.chunk().await {
                    file.write_all(&chunk).await?;
                }
            }
        }
    }
    Ok(Json(json!({"image_path": "/".to_owned() + &*image_path})))
}