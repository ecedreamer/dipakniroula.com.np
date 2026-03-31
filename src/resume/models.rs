use chrono::{Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::{Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::experiences)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Experience {
    pub id: Option<i32>,
    pub company_name: String,
    pub your_position: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub responsibility: Option<String>,
    pub skills: Option<String>,
    pub company_link: String,
    pub order: i32,
}

impl Experience {
    pub fn duration(&self) -> String {
        let start = match NaiveDate::parse_from_str(&self.start_date, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => return "".to_string(),
        };

        let end = match &self.end_date {
            Some(date_str) if !date_str.is_empty() => {
                match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    Ok(date) => date,
                    Err(_) => Utc::now().naive_utc().date(),
                }
            }
            _ => Utc::now().naive_utc().date(),
        };

        if end < start {
            return "".to_string();
        }

        let mut years = end.year() - start.year();
        let mut months = end.month() as i32 - start.month() as i32;

        if months < 0 {
            years -= 1;
            months += 12;
        }

        match (years, months) {
            (0, 0) => "".to_string(),
            (y, 0) if y > 0 => format!("( {} year{})", y, if y > 1 { "s" } else { "" }),
            (0, m) if m > 0 => format!("( {} month{})", m, if m > 1 { "s" } else { "" }),
            (y, m) => format!(
                "( {} year{} {} month{})",
                y,
                if y > 1 { "s" } else { "" },
                m,
                if m > 1 { "s" } else { "" }
            ),
        }
    }
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::experiences)]
pub struct NewExperience {
    pub company_name: String,
    pub company_link: String,
    pub your_position: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub responsibility: Option<String>,
    pub skills: Option<String>,
    pub order: i32,
}

#[derive(Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::experiences)]
pub struct UpdateExperience {
    pub company_name: Option<String>,
    pub company_link: Option<String>,
    pub your_position: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub responsibility: Option<String>,
    pub skills: Option<String>,
    pub order: Option<i32>,
}
