use diesel::prelude::*;
use uuid::Uuid;
use chrono::Utc;
use crate::models::CustomSession;


pub fn create_session(conn: &mut SqliteConnection, u_id: String) -> QueryResult<CustomSession> {
    use crate::schema::sessions::dsl::*;

    let new_session = CustomSession {
        id: None, // Will be auto-incremented
        session_id: Uuid::new_v4().to_string(),
        user_id: u_id,
        data: None,
        expires_at: Utc::now().naive_utc() + chrono::Duration::hours(60), // 1-hour expiry
    };

    diesel::insert_into(sessions)
        .values(&new_session)
        .execute(conn)?;

    sessions
        .order(id.desc())
        .first::<CustomSession>(conn)
}


pub fn get_session(conn: &mut SqliteConnection, sess_id: &str) -> QueryResult<Option<CustomSession>> {
    use crate::schema::sessions::dsl::*;

    sessions
        .filter(session_id.eq(sess_id))
        .first::<CustomSession>(conn)
        .optional()
}

pub fn delete_expired_sessions(conn: &mut SqliteConnection) -> QueryResult<usize> {
    use crate::schema::sessions::dsl::*;
    diesel::delete(sessions.filter(expires_at.lt(Utc::now().naive_utc()))).execute(conn)
}