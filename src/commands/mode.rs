use serenity::all::{CommandInteraction, CommandOptionType, Permissions, PermissionOverwrite, PermissionOverwriteType};
use crate::database::get_channel_by_id;
use crate::profile::Profile;
use crate::*;
use serenity::builder::{CreateCommand, CreateCommandOption, EditInteractionResponse};
use serenity::client::Context;

pub fn register() -> CreateCommand {
    CreateCommand::new("mode")
        .description("Change how everyone can interact with this channel")
        .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "public", "Everyone can read and write messages"))
        .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "visible", "Everyone can read messages, but only added users can write messages"))
        .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "private", "Only added users can read and/or write messages"))
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let _ = cmd.defer(&ctx).await;

    let data = ctx.data.read().await;
    let profile = data.get::<Profile>().unwrap();
    let database_pool = data.get::<DatabasePoolKey>().unwrap();

    if let Ok(channel) = get_channel_by_id(database_pool.clone(), cmd.channel_id).await {
        if channel.owner == cmd.user.id {
            if let Ok(discord_channel) = cmd.channel_id.to_channel(&ctx).await {
                let discord_channel = discord_channel.guild().unwrap();

                let new_mode;
                match cmd.data.options[0].name.as_str() {
                    "public" => {
                        new_mode = "fully public";
                        let _  = discord_channel.create_permission(&ctx, channel_public!(profile.roles.member())).await;
                    }
                    "visible" => {
                        new_mode = "visible to everyone";
                        let _  = discord_channel.create_permission(&ctx, channel_viewable!(profile.roles.member())).await;
                    }
                    "private" => {
                        new_mode = "private";
                        let _ = discord_channel.delete_permission(&ctx, PermissionOverwriteType::Role(profile.roles.member())).await;
                    }
                    _ => {
                        unreachable!("Invalid mode");
                    }
                }

                let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                    .content(format!("The channel is now {new_mode}"))).await;
            } else {
                let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                    .content("Failed to retrieve channel data")).await;
            }
        } else {
            let _ = cmd.delete_response(&ctx).await;
        }
    } else {
        let _ = cmd.delete_response(&ctx).await;
    }
}