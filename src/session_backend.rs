use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::AsyncPgConnection;
use uuid::Uuid;
use chrono::Utc;
use crate::models::CustomSession;

pub async fn create_session(conn: &mut AsyncPgConnection, u_id: String) -> QueryResult<CustomSession> {
    use crate::schema::sessions::dsl::*;

    let new_session = CustomSession {
        id: None, // Will be auto-incremented
        session_id: Uuid::new_v4().to_string(),
        user_id: u_id,
        data: None,
        expires_at: Utc::now().naive_utc() + chrono::Duration::try_hours(60).unwrap(), // 1-hour expiry
    };

    diesel::insert_into(sessions)
        .values(&new_session)
        .execute(conn)
        .await?;

    sessions
        .order(id.desc())
        .first::<CustomSession>(conn)
        .await
}

pub async fn get_session(conn: &mut AsyncPgConnection, sess_id: &str) -> QueryResult<Option<CustomSession>> {
    use crate::schema::sessions::dsl::*;

    sessions
        .filter(session_id.eq(sess_id))
        .first::<CustomSession>(conn)
        .await
        .optional()
}

pub async fn delete_expired_sessions(conn: &mut AsyncPgConnection) -> QueryResult<usize> {
    use crate::schema::sessions::dsl::*;
    diesel::delete(sessions.filter(expires_at.lt(Utc::now().naive_utc())))
        .execute(conn)
        .await
}