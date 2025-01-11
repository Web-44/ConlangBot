pub mod channel;
pub mod profile;
pub mod database;
pub mod handler;
pub mod commands;

use serenity::all::{ActivityData, GatewayIntents, OnlineStatus};
use serenity::Client;
use std::env;
use std::sync::Arc;
use serenity::gateway::ShardManager;
use serenity::prelude::TypeMapKey;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::MySqlPool;
use crate::database::SqlPool;
use crate::handler::Handler;
use crate::profile::{read_profile, Profile};

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Env missing DISCORD_TOKEN");
    let app_id = env::var("DISCORD_APP_ID").expect("Env missing DISCORD_APP_ID");
    let app_id = app_id.parse().expect("DISCORD_APP_ID is not a number");

    let sql_host = env::var("SQL_HOST").expect("Env missing SQL_HOST");
    let sql_port = env::var("SQL_PORT").expect("Env missing SQL_PORT");
    let sql_port = sql_port.parse().expect("SQL_PORT is not a number");
    let sql_database = env::var("SQL_DATABASE").expect("Env missing SQL_DATABASE");
    let sql_username = env::var("SQL_USERNAME").expect("Env missing SQL_USERNAME");
    let sql_password = env::var("SQL_PASSWORD").expect("Env missing SQL_PASSWORD");

    let profile = env::var("PROFILE_PATH").expect("Env missing PROFILE_PATH");
    let profile = read_profile(profile);

    let mut client = Client::builder(&token, GatewayIntents::GUILDS)
        .application_id(app_id)
        .event_handler(Handler)
        .activity(ActivityData::playing("Conlanging"))
        .status(OnlineStatus::Online)
        .await
        .expect("Error creating client");

    let database_pool = MySqlPool::connect_with(MySqlConnectOptions::new()
        .host(&sql_host)
        .port(sql_port)
        .database(&sql_database)
        .username(&sql_username)
        .password(&sql_password))
        .await
        .expect("Failed to connect to database");

    {
        let mut data = client.data.write().await;
        data.insert::<Profile>(profile);
        data.insert::<ShardManagerKey>(client.shard_manager.clone());
        data.insert::<DatabasePoolKey>(Arc::new(database_pool));
    }

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
}

pub struct ShardManagerKey;
impl TypeMapKey for ShardManagerKey {
    type Value = Arc<ShardManager>;
}

pub struct DatabasePoolKey;
impl TypeMapKey for DatabasePoolKey {
    type Value = Arc<SqlPool>;
}