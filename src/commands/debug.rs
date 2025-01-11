use crate::profile::Profile;
use serenity::all::{CommandInteraction, Context};
use serenity::builder::{CreateCommand, CreateEmbed, EditInteractionResponse};
use serenity::model::Permissions;
use std::env;
use crate::commands::DEVELOPER;
use crate::ShardManagerKey;

pub fn register() -> CreateCommand {
    CreateCommand::new("debug")
        .description("Get information about the bot instance")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .dm_permission(true)
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let _ = cmd.defer(&ctx).await;

    if cmd.user.id != DEVELOPER {
        let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
            .content("You are not allowed to use this command")).await;
        return;
    }

    let data = ctx.data.read().await;
    let profile = data.get::<Profile>().unwrap();
    let shard_manager = data.get::<ShardManagerKey>().unwrap();
    let shard_runners = shard_manager.runners.lock().await;

    for (id, runner) in shard_runners.iter() {
        println!(
            "Shard ID {} is {} with a latency of {:?}",
            id, runner.stage, runner.latency,
        );
    }

    let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
        .add_embed(CreateEmbed::new()
            .title("Bot Information")
            .description("Shard infos printed to console")
            .field("Version", env!("CARGO_PKG_VERSION"), true)
            .field("Build Script", env!("CH_BUILDSCRIPT"), true)
            .field("Build Profile", env!("CH_PROFILE"), true)
            .field("Build Host", env!("CH_HOST"), false)
            .field("Build Target", env!("CH_TARGET"), false)
            .field("Profile Name", profile.name.as_str(), true)
            .field("Profile Path", env::var("PROFILE_PATH").unwrap(), true))).await;
}