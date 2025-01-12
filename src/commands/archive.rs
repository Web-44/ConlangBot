use crate::database::get_channel_by_id;
use crate::profile::Profile;
use crate::DatabasePoolKey;
use serenity::all::{ChannelId, CommandInteraction, Context, CreateCommand};
use serenity::builder::{EditChannel, EditInteractionResponse};
use crate::channel::{get_channels_in_category, sort_category};

pub fn register() -> CreateCommand {
    CreateCommand::new("archive")
        .description("Archives/Unarchives the channel")
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let _ = cmd.defer_ephemeral(&ctx).await;

    let data = ctx.data.read().await;
    let profile = data.get::<Profile>().unwrap();
    let database_pool = data.get::<DatabasePoolKey>().unwrap();

    if let Ok(channel) = get_channel_by_id(database_pool.clone(), cmd.channel_id).await {
        if channel.check_permission(&cmd.user, &cmd.member) {
            if let Ok(discord_channel) = cmd.channel_id.to_channel(&ctx).await {
                let mut discord_channel = discord_channel.guild().unwrap();

                if let Some(parent_id) = discord_channel.parent_id {
                    if profile.is_archive(parent_id) {
                        if let Some(category) = channel.category {
                            let _ = discord_channel.edit(&ctx, EditChannel::new().category(channel.category())).await;
                            let _ = sort_category(category, &ctx).await;
                            let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                                .content("The channel has been unarchived")).await;
                        } else {
                            let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                                .content("The channel is not yet assigned a category. Please use /category to select one first!")).await;
                        }
                    } else {
                        for archive in &profile.archives {
                            if let Ok(channels) = get_channels_in_category(*archive, &ctx).await {
                                if channels.len() < 50 {
                                    let _ = discord_channel.edit(&ctx, EditChannel::new().category(Some(ChannelId::new(*archive)))).await;
                                    let _ = sort_category(*archive, &ctx).await;
                                    let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                                        .content("The channel has been archived")).await;
                                    break;
                                }
                            }
                        }
                    }
                }
            } else {
                let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                    .content("Failed to retrieve channel data")).await;
            }
        } else {
            let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                .content("You can only delete your own channel")).await;
        }
    } else {
        let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
            .content("Could not find channel data")).await;
    }
}