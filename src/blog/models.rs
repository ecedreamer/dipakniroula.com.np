use diesel::{AsChangeset, Insertable, Queryable};
use serde::Deserialize;
use std::sync::atomic::{AtomicU64, Ordering};

static SLUG_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn generate_slug(title: &str) -> String {
    let base: String = title
        .to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() || c == '-' || c == ' ' {
                Some(if c == ' ' { '-' } else { c })
            } else if c == '_' || c == '.' {
                Some(c)
            } else {
                None
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .trim_matches('-')
        .to_string();

    if base.is_empty() {
        return format!("post-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
    }

    let truncated: String = base.chars().take(80).collect();
    let counter = SLUG_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}", truncated, counter % 99999)
}


#[derive(Debug, Queryable, Deserialize)]
#[diesel(table_name = crate::schema::blogs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Blog {
    pub id: Option<i32>,
    pub title: String,
    pub content: String,
    pub image: Option<String>,
    pub published_date: String,
    pub modified_date: Option<String>,
    pub view_count: i32,
    pub is_active: i32,
    pub slug: String,
}

impl Blog {
    fn strip_html(&self) -> String {
        self.content
            .chars()
            .fold((String::new(), false), |(mut acc, mut in_tag), c| {
                if c == '<' {
                    in_tag = true;
                } else if c == '>' {
                    in_tag = false;
                } else if !in_tag {
                    acc.push(c);
                }
                (acc, in_tag)
            })
            .0
    }

    pub fn reading_time_minutes(&self) -> i32 {
        let stripped = self.strip_html();
        let word_count = stripped.split_whitespace().count() as f64;
        let minutes = (word_count / 200.0).ceil() as i32;
        minutes.max(1)
    }

    pub fn excerpt(&self, max_chars: usize) -> String {
        let stripped = self.strip_html();
        let decoded = stripped
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&apos;", "'");
        if decoded.chars().count() <= max_chars {
            decoded
        } else {
            let truncated: String = decoded.chars().take(max_chars).collect();
            let last_space = truncated.rfind(' ').unwrap_or(max_chars);
            format!("{}...", &truncated[..last_space])
        }
    }
}


#[derive(Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::blogs)]
pub struct UpdateBlog {
    pub is_active: Option<i32>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub image: Option<String>,
    pub modified_date: Option<String>,
    pub view_count: Option<i32>,
    pub slug: Option<String>,
}


#[derive(Insertable)]
#[diesel(table_name = crate::schema::blogs)]
pub struct NewBlog<'a> {
    pub title: &'a str,
    pub content: &'a str,
    pub image: Option<&'a str>,
    pub is_active: i32,
    pub published_date: String,
    pub modified_date: Option<String>,
    pub slug: &'a str,
}


#[derive(Queryable, Deserialize, Debug)]
#[diesel(table_name = crate::schema::categories)]
pub struct Category {
    pub id: i32,
    pub name: String,
}

// Join table for the many-to-many relationship between blogs and tags
#[derive(Queryable, Insertable, Debug)]
#[diesel(table_name = crate::schema::blog_categories)]
pub struct BlogCategory {
    pub blog_id: i32,
    pub category_id: i32,
}


#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::categories)]
pub struct NewCategory {
    pub name: String
}