use crate::database::get_channel_by_id;
use crate::DatabasePoolKey;
use serenity::all::{CommandInteraction, Context, CreateCommand, InputTextStyle};
use serenity::builder::{CreateActionRow, CreateInputText, CreateInteractionResponse, CreateInteractionResponseMessage, CreateModal};

pub fn register() -> CreateCommand {
    CreateCommand::new("edit")
        .description("Edit the channel name and/or topic")
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let data = ctx.data.read().await;
    let database_pool = data.get::<DatabasePoolKey>().unwrap();

    if let Ok(channel) = get_channel_by_id(database_pool.clone(), cmd.channel_id).await {
        if channel.check_permission(&cmd.user, &cmd.member) {
            if let Ok(discord_channel) = cmd.channel_id.to_channel(&ctx).await {
                let discord_channel = discord_channel.guild().unwrap();
                let _ = cmd.create_response(&ctx, CreateInteractionResponse::Modal(
                    CreateModal::new("edit-channel", "Create Channel")
                        .components(vec![
                            CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Short, "Name", "0")
                                .required(true)
                                .min_length(2)
                                .max_length(100)
                                .value(discord_channel.name)),
                            CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Paragraph, "Topic", "1")
                                .required(false)
                                .max_length(1024)
                                .value(discord_channel.topic.unwrap_or("".to_string())))
                        ])
                )).await;
            } else {
                let _ = cmd.create_response(&ctx, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                    .content("Failed to retrieve channel data")
                    .ephemeral(true))).await;
            }
        } else {
            let _ = cmd.create_response(&ctx, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                .content("You can only edit your own channel")
                .ephemeral(true))).await;
        }
    } else {
        let _ = cmd.create_response(&ctx, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
            .content("Could not find channel data")
            .ephemeral(true))).await;
    }
}