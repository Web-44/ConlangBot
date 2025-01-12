use crate::profile::Profile;
use serenity::all::{Command, GuildChannel, Interaction, Message, RatelimitInfo, Ready};
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use crate::database::delete_channel_by_id;
use crate::DatabasePoolKey;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn channel_delete(&self, ctx: Context, channel: GuildChannel, _messages: Option<Vec<Message>>) {
        let data = ctx.data.read().await;
        let database_pool = data.get::<DatabasePoolKey>().expect("Failed to get database pool");

        if let Err(err) = delete_channel_by_id(database_pool.clone(), channel.id).await {
            eprintln!("Failed to delete channel #{} from database: {err:?}", channel.id);
        }
    }

    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        let data = ctx.data.read().await;
        let profile = data.get::<Profile>().expect("Failed to get profile");

        if let Ok(val) = std::env::var("UPDATECMD") {
            if val.parse().unwrap_or(false) {
                profile.guild().set_commands(&ctx, vec![
                    crate::commands::archive::register(),
                    crate::commands::ban::register(),
                    crate::commands::category::register(profile),
                    crate::commands::contributor::register(),
                    crate::commands::create::register(),
                    crate::commands::delete::register(),
                    crate::commands::edit::register(),
                    crate::commands::fixperms::register(),
                    crate::commands::migrate::register(),
                    crate::commands::mode::register(),
                    crate::commands::unban::register(),
                    crate::commands::viewer::register(),
                    crate::commands::wordgen::register(),
                ]).await.expect("Failed to set guild commands");

                Command::create_global_command(&ctx, crate::commands::debug::register()).await.expect("Failed to set global command: debug");
            }
        }

        println!("ConlangBot started");
    }

    async fn ratelimit(&self, data: RatelimitInfo) {
        if data.global {
            println!("[Ratelimit] Global on {:?} {} (limit {}): {} seconds", data.method, data.path, data.limit, data.timeout.as_secs());
        } else {
            println!("[Ratelimit] Local on {:?} {} (limit {}): {} seconds", data.method, data.path, data.limit, data.timeout.as_secs());
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let data = ctx.data.read().await;
        let profile = data.get::<Profile>().unwrap();

        match interaction {
            Interaction::Command(cmd) => {
                if let Some(guild_id) = cmd.guild_id {
                    if guild_id.get() != profile.guild {
                        return;
                    }
                }
                match cmd.data.name.as_str() {
                    "archive" => crate::commands::archive::run(&ctx, cmd).await,
                    "ban" => crate::commands::ban::run(&ctx, cmd).await,
                    "category" => crate::commands::category::run(&ctx, cmd).await,
                    "contributor" => crate::commands::contributor::run(&ctx, cmd).await,
                    "create" => crate::commands::create::run(&ctx, cmd).await,
                    "debug" => crate::commands::debug::run(&ctx, cmd).await,
                    "delete" => crate::commands::delete::run(&ctx, cmd).await,
                    "edit" => crate::commands::edit::run(&ctx, cmd).await,
                    "fixperms" => crate::commands::fixperms::run(&ctx, cmd).await,
                    "migrate" => crate::commands::migrate::run(&ctx, cmd).await,
                    "mode" => crate::commands::mode::run(&ctx, cmd).await,
                    "unban" => crate::commands::unban::run(&ctx, cmd).await,
                    "viewer" => crate::commands::viewer::run(&ctx, cmd).await,
                    "wordgen" => crate::commands::wordgen::run(&ctx, cmd).await,
                    _ => {}
                }
            }
            Interaction::Component(comp) => {
                if let Some(guild_id) = comp.guild_id {
                    if guild_id.get() != profile.guild {
                        return;
                    }
                }

                let id = comp.data.custom_id.as_str();
                if id.starts_with("create-channel") {
                    crate::commands::create_interaction::run(&ctx, comp).await;
                } else if id == "delete-channel" {
                    crate::commands::delete_interaction::run(&ctx, comp).await;
                }
            }
            Interaction::Modal(modal) => {
                if let Some(guild_id) = modal.guild_id {
                    if guild_id.get() != profile.guild {
                        return;
                    }
                }

                let id = modal.data.custom_id.as_str();
                if id.starts_with("create-channel") {
                    crate::commands::create_modal::run(&ctx, modal).await;
                } else if id == "edit-channel" {
                    crate::commands::edit_modal::run(&ctx, modal).await;
                }
            }
            _ => {}
        }
    }
}