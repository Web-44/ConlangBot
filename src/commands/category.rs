use crate::database::{edit_channel, get_channel_by_id};
use crate::*;
use serenity::all::{ChannelId, CommandInteraction, CommandOptionType, Context};
use serenity::builder::{CreateCommand, CreateCommandOption, EditChannel, EditInteractionResponse};
use crate::channel::sort_category;

pub fn register(profile: &Profile) -> CreateCommand {
    CreateCommand::new("category")
        .description("Change channel category")
        .set_options(profile.categories.iter().map(|cat| {
            CreateCommandOption::new(CommandOptionType::SubCommand, cat.name.to_lowercase(), format!("Move channel to {}", cat.name))
        }).collect())
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let _ = cmd.defer_ephemeral(&ctx).await;

    let data = ctx.data.read().await;
    let profile = data.get::<Profile>().unwrap();
    let database_pool = data.get::<DatabasePoolKey>().unwrap();

    if let Ok(mut channel) = get_channel_by_id(database_pool.clone(), cmd.channel_id).await {
        if channel.owner == cmd.user.id {
            let name = cmd.data.options[0].name.as_str();
            if let Some(category) = profile.categories.iter().find(|cat| cat.name.to_lowercase().as_str() == name) {
                if let Ok(discord_channel) = cmd.channel_id.to_channel(&ctx).await {
                    let mut discord_channel = discord_channel.guild().unwrap();

                    let old_category = channel.category;
                    channel.category = Some(category.id);
                    if let Err(err) = edit_channel(database_pool.clone(), channel).await {
                        let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                            .content(format!("Failed to edit channel data: {err}"))).await;
                    } else {
                        if let Some(old_category) = old_category {
                            if discord_channel.parent_id == Some(ChannelId::new(old_category)) {
                                let _ = discord_channel.edit(&ctx, EditChannel::new().category(Some(ChannelId::new(category.id)))).await;
                                let _ = sort_category(category.id, &ctx).await;
                            }
                        }
                        let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                            .content(format!("The channel now belongs to the {} category", category.name))).await;
                    }
                } else {
                    let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                        .content("Failed to retrieve channel data")).await;
                }
            } else {
                let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                    .content("No such category")).await;
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