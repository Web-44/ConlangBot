use crate::database::get_channel_by_id;
use crate::*;
use serenity::all::{CommandDataOptionValue, CommandInteraction, CommandOptionType, Context, Permissions, PermissionOverwrite, PermissionOverwriteType};
use serenity::builder::{CreateAllowedMentions, CreateCommand, CreateCommandOption, EditInteractionResponse};

pub fn register() -> CreateCommand {
    CreateCommand::new("viewer")
        .description("Add/Remove users that can view this channel")
        .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "add", "Add a viewer")
            .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "user", "The user to add").required(true)))
        .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "remove", "Remove a viewer")
            .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "user", "The user to remove").required(true)))
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let _ = cmd.defer(&ctx).await;

    let data = ctx.data.read().await;
    let database_pool = data.get::<DatabasePoolKey>().unwrap();

    if let Ok(channel) = get_channel_by_id(database_pool.clone(), cmd.channel_id).await {
        if channel.check_permission(&cmd.user, &cmd.member) {
            if let CommandDataOptionValue::SubCommand(options) = &cmd.data.options[0].value {
                if let Some(user) = options[0].value.as_user_id() {
                    if channel.owner == user {
                        let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                            .content("You can't change the channel owner's permissions")).await;
                        return;
                    }
                    if let Ok(user) = user.to_user(&ctx).await {
                        if user.bot {
                            let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                                .content("You can't change a bot's permissions")).await;
                            return;
                        }
                    } else {
                        let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                            .content("Failed to retrieve user data")).await;
                        return;
                    }

                    if let Ok(discord_channel) = cmd.channel_id.to_channel(&ctx).await {
                        let discord_channel = discord_channel.guild().unwrap();

                        match cmd.data.options[0].name.as_str() {
                            "add" => {
                                let _ = discord_channel.create_permission(&ctx, user_viewer!(user)).await;
                                let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                                    .content(format!("Viewer added: <@{user}>"))
                                    .allowed_mentions(CreateAllowedMentions::new())).await;
                            }
                            "remove" => {
                                let _ = discord_channel.delete_permission(&ctx, PermissionOverwriteType::Member(user)).await;
                                let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                                    .content(format!("Viewer removed: <@{user}>"))
                                    .allowed_mentions(CreateAllowedMentions::new())).await;
                            }
                            _ => {}
                        }
                    } else {
                        let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                            .content("Failed to retrieve channel data")).await;
                    }
                }
            }
        } else {
            let _ = cmd.delete_response(&ctx).await;
        }
    } else {
        let _ = cmd.delete_response(&ctx).await;
    }
}