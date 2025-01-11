use std::sync::Arc;
use serenity::all::{ChannelId, UserId};
use sqlx::{query, query_as, Error, FromRow, MySqlPool, Row};
use sqlx::mysql::MySqlRow;
use crate::channel::ConChannel;

pub type SqlPool = MySqlPool;
pub type SqlRow = MySqlRow;
pub type SqlResult<T> = Result<T, Error>;

impl FromRow<'_, SqlRow> for ConChannel {
    fn from_row(row: &SqlRow) -> SqlResult<ConChannel> {
        let id = ChannelId::new(row.get(0));
        let owner = UserId::new(row.get(1));
        let category = row.get(2);

        Ok(ConChannel { id, owner, category })
    }
}

pub async fn add_channel(pool: Arc<SqlPool>, channel: ConChannel) -> SqlResult<()> {
    query("INSERT INTO Channels (ID, Owner, Category) VALUES (?, ?, ?)")
        .bind(channel.id.get())
        .bind(channel.owner.get())
        .bind(channel.category)
        .execute(&*pool)
        .await?;

    Ok(())
}

pub async fn edit_channel(pool: Arc<SqlPool>, channel: ConChannel) -> SqlResult<()> {
    query("UPDATE Channels SET Owner = ?, Category = ? WHERE ID = ?")
        .bind(channel.owner.get())
        .bind(channel.category)
        .bind(channel.id.get())
        .execute(&*pool)
        .await?;

    Ok(())
}

pub async fn get_channel_by_id(pool: Arc<SqlPool>, id: ChannelId) -> SqlResult<ConChannel> {
    query_as("SELECT * FROM Channels WHERE ID = ?")
        .bind(id.get())
        .fetch_one(&*pool)
        .await
}

pub async fn get_channels_by_owner(pool: Arc<SqlPool>, id: UserId) -> SqlResult<Vec<ConChannel>> {
    query_as("SELECT * FROM Channels WHERE Owner = ?")
        .bind(id.get())
        .fetch_all(&*pool)
        .await
}

pub async fn delete_channel_by_id(pool: Arc<SqlPool>, id: ChannelId) -> SqlResult<ConChannel> {
    query_as("DELETE FROM Channels WHERE ID = ?")
        .bind(id.get())
        .fetch_one(&*pool)
        .await
}