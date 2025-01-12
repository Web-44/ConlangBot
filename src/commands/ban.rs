use crate::database::get_channel_by_id;
use crate::*;
use serenity::all::{CommandDataOptionValue, CommandInteraction, CommandOptionType, Context, PermissionOverwrite, PermissionOverwriteType, Permissions};
use serenity::builder::{CreateAllowedMentions, CreateCommand, CreateCommandOption, EditInteractionResponse};

pub fn register() -> CreateCommand {
    CreateCommand::new("ban")
        .description("Ban users from your channel")
        .add_option(CreateCommandOption::new(CommandOptionType::User, "user", "The user to ban").required(true))
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let _ = cmd.defer(&ctx).await;

    let data = ctx.data.read().await;
    let database_pool = data.get::<DatabasePoolKey>().unwrap();

    if let Ok(channel) = get_channel_by_id(database_pool.clone(), cmd.channel_id).await {
        if channel.check_permission(&cmd.user, &cmd.member) {
            if let CommandDataOptionValue::User(user) = cmd.data.options[0].value {
                if channel.owner == user {
                    let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                        .content("You can't ban the channel owner")).await;
                    return;
                }
                if let Ok(user) = user.to_user(&ctx).await {
                    if user.bot {
                        let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                            .content("You can't ban a bot")).await;
                        return;
                    }
                } else {
                    let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                        .content("Failed to retrieve user data")).await;
                    return;
                }

                if let Ok(discord_channel) = cmd.channel_id.to_channel(&ctx).await {
                    let discord_channel = discord_channel.guild().unwrap();

                    let _ = discord_channel.create_permission(&ctx, user_banned!(user)).await;
                    let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                        .content(format!("User banned: <@{user}>"))
                        .allowed_mentions(CreateAllowedMentions::new())).await;
                } else {
                    let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                        .content("Failed to retrieve channel data")).await;
                }
            }
        } else {
            let _ = cmd.delete_response(&ctx).await;
        }
    } else {
        let _ = cmd.delete_response(&ctx).await;
    }
}