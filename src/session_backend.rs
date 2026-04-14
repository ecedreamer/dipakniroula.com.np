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

pub async fn set_flash(
    conn: &mut AsyncPgConnection,
    sess: &CustomSession,
    success: Option<String>,
    error: Option<String>,
) -> QueryResult<()> {
    use crate::schema::sessions::dsl::*;
    let flash = crate::models::FlashData { success, error };
    let json_data = serde_json::to_string(&flash).unwrap_or_default();

    diesel::update(sessions.filter(id.eq(sess.id.unwrap())))
        .set(data.eq(Some(json_data)))
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn take_flash(
    conn: &mut AsyncPgConnection,
    sess: &CustomSession,
) -> (Option<CustomSession>, crate::models::FlashData) {
    use crate::schema::sessions::dsl::*;
    
    let flash = if let Some(ref d) = sess.data {
        serde_json::from_str::<crate::models::FlashData>(d).unwrap_or_default()
    } else {
        crate::models::FlashData::default()
    };

    // Clear the data in the DB
    let update_res = diesel::update(sessions.filter(id.eq(sess.id.unwrap())))
        .set(data.eq(None::<String>))
        .get_result::<CustomSession>(conn)
        .await;

    (update_res.ok(), flash)
}