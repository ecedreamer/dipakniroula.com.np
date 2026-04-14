use crate::message::models::Message;
use crate::schema::messages;
use anyhow::{Result, anyhow};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

pub struct MessageRepository<'a> {
    conn: &'a mut AsyncPgConnection,
}

impl<'a> MessageRepository<'a> {
    pub fn new(conn: &'a mut AsyncPgConnection) -> Self {
        MessageRepository { conn }
    }

    pub async fn find_all(&mut self, limit: i64, offset: i64) -> Result<Vec<Message>> {
        let messages = messages::table
            .limit(limit)
            .offset(offset)
            .order(messages::date_sent.desc())
            .select(Message::as_select())
            .load(self.conn)
            .await?;
        Ok(messages)
    }

    pub async fn count_all(&mut self) -> Result<i64> {
        let count = messages::table.count().get_result(self.conn).await?;
        Ok(count)
    }

    pub async fn find_by_id(&mut self, message_id: i32) -> Result<Message> {
        let message = messages::table
            .filter(messages::id.eq(message_id))
            .select(Message::as_select())
            .first(self.conn)
            .await?;
        Ok(message)
    }

    pub async fn delete_one(&mut self, message_id: i32) -> Result<()> {
        diesel::delete(messages::table.filter(messages::id.eq(message_id)))
            .execute(self.conn)
            .await?;
        Ok(())
    }

    pub async fn delete_multiple(&mut self, message_ids: Vec<i32>) -> Result<()> {
        diesel::delete(messages::table.filter(messages::id.is_in(message_ids)))
            .execute(self.conn)
            .await?;
        Ok(())
    }
}
