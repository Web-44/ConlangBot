use crate::profile::Profile;
use crate::*;
use serenity::all::{CommandInteraction, PermissionOverwrite, PermissionOverwriteType, Permissions};
use serenity::builder::{CreateCommand, EditInteractionResponse};
use serenity::client::Context;
use crate::database::get_channel_by_id;

pub fn register() -> CreateCommand {
    CreateCommand::new("fixperms")
        .description("Fix channel permissions of owner and @everyone")
        .default_member_permissions(Permissions::MANAGE_CHANNELS)
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let _ = cmd.defer_ephemeral(&ctx).await;

    let data = ctx.data.read().await;
    let database_pool = data.get::<DatabasePoolKey>().unwrap();
    let profile = data.get::<Profile>().unwrap();

    if let Ok(channel) = get_channel_by_id(database_pool.clone(), cmd.channel_id).await {
        if let Ok(discord_channel) = cmd.channel_id.to_channel(&ctx).await {
            let discord_channel = discord_channel.guild().unwrap();
            let _ = discord_channel.create_permission(&ctx, hide_to_everyone!(profile.roles.everyone())).await;
            let _ = discord_channel.create_permission(&ctx, user_owner!(channel.owner)).await;
            let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                .content("Fixed permissions")).await;
        } else {
            let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                .content("Failed to retrieve channel data")).await;
        }
    } else {
        let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
            .content("Could not find channel data")).await;
    }
}