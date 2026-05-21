use diesel::prelude::*;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;

use super::models::{AppSetting, NewQuizAttempt, NewQuizSession, QuizAttempt, QuizSession};

pub struct QuizRepository<'a> {
    pub conn: &'a mut AsyncPgConnection,
}

impl<'a> QuizRepository<'a> {
    pub fn new(conn: &'a mut AsyncPgConnection) -> Self {
        Self { conn }
    }

    pub async fn get_setting(self, setting_key: &str) -> QueryResult<Option<String>> {
        use crate::schema::app_settings::dsl::*;

        app_settings
            .filter(key.eq(setting_key))
            .select(value)
            .first::<String>(self.conn)
            .await
            .optional()
    }

    pub async fn upsert_setting(self, setting_key: &str, setting_value: &str) -> QueryResult<()> {
        use crate::schema::app_settings::dsl::*;

        let existing = app_settings
            .filter(key.eq(setting_key))
            .first::<AppSetting>(self.conn)
            .await
            .optional()?;

        if existing.is_some() {
            diesel::update(app_settings.filter(key.eq(setting_key)))
                .set(value.eq(setting_value))
                .execute(self.conn)
                .await?;
        } else {
            diesel::insert_into(app_settings)
                .values((key.eq(setting_key), value.eq(setting_value)))
                .execute(self.conn)
                .await?;
        }

        Ok(())
    }

    pub async fn insert_quiz_session(self, data: &NewQuizSession) -> QueryResult<QuizSession> {
        use crate::schema::quiz_sessions::dsl::*;

        diesel::insert_into(quiz_sessions)
            .values(data)
            .execute(self.conn)
            .await?;

        quiz_sessions
            .order(id.desc())
            .first::<QuizSession>(self.conn)
            .await
    }

    pub async fn find_quiz_session_by_uuid(
        self,
        uuid: &str,
    ) -> QueryResult<Option<QuizSession>> {
        use crate::schema::quiz_sessions::dsl::*;

        quiz_sessions
            .filter(session_uuid.eq(uuid))
            .first::<QuizSession>(self.conn)
            .await
            .optional()
    }

    pub async fn delete_quiz_session(self, uuid: &str) -> QueryResult<usize> {
        use crate::schema::quiz_sessions::dsl::*;

        diesel::delete(quiz_sessions.filter(session_uuid.eq(uuid)))
            .execute(self.conn)
            .await
    }

    pub async fn insert_attempt(self, data: &NewQuizAttempt) -> QueryResult<QuizAttempt> {
        use crate::schema::quiz_attempts::dsl::*;

        diesel::insert_into(quiz_attempts)
            .values(data)
            .execute(self.conn)
            .await?;

        quiz_attempts
            .order(id.desc())
            .first::<QuizAttempt>(self.conn)
            .await
    }

    pub async fn find_all(
        self,
        page: i64,
        per_page: i64,
    ) -> QueryResult<(Vec<QuizAttempt>, i64)> {
        use crate::schema::quiz_attempts::dsl::*;

        let total = quiz_attempts
            .count()
            .get_result::<i64>(self.conn)
            .await
            .unwrap_or(0);

        let offset = (page - 1) * per_page;
        let results = quiz_attempts
            .order(id.desc())
            .limit(per_page)
            .offset(offset)
            .load::<QuizAttempt>(self.conn)
            .await?;

        Ok((results, total))
    }

    pub async fn find_by_id(self, attempt_id: i32) -> QueryResult<QuizAttempt> {
        use crate::schema::quiz_attempts::dsl::*;

        quiz_attempts
            .filter(id.eq(attempt_id))
            .first::<QuizAttempt>(self.conn)
            .await
    }

    pub async fn count_all(self) -> QueryResult<i64> {
        use crate::schema::quiz_attempts::dsl::*;

        quiz_attempts
            .count()
            .get_result::<i64>(self.conn)
            .await
    }
}
