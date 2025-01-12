use crate::database::get_channel_by_id;
use crate::DatabasePoolKey;
use serenity::all::{ButtonStyle, CommandInteraction, Context, CreateCommand};
use serenity::builder::{CreateActionRow, CreateButton, EditInteractionResponse};

pub fn register() -> CreateCommand {
    CreateCommand::new("delete")
        .description("Deletes the channel")
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let _ = cmd.defer_ephemeral(&ctx).await;

    let data = ctx.data.read().await;
    let database_pool = data.get::<DatabasePoolKey>().unwrap();

    if let Ok(channel) = get_channel_by_id(database_pool.clone(), cmd.channel_id).await {
        if channel.check_permission(&cmd.user, &cmd.member) {
            let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                .content("Are you sure you want to delete this channel?")
                .components(vec![
                    CreateActionRow::Buttons(vec![
                        CreateButton::new("delete-channel")
                            .label("Yes, delete")
                            .style(ButtonStyle::Danger)
                    ])
                ])).await;
        } else {
            let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                .content("You can only delete your own channel")).await;
        }
    } else {
        let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
            .content("Could not find channel data")).await;
    }
}