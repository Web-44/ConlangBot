use serenity::all::{ButtonStyle, CommandInteraction};
use serenity::builder::{CreateActionRow, CreateButton, CreateCommand, CreateEmbed, CreateMessage, EditInteractionResponse};
use serenity::client::Context;
use serenity::model::Permissions;
use crate::profile::Profile;

pub fn register() -> CreateCommand {
    CreateCommand::new("create")
        .description("Send the channel create message")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let _ = cmd.defer_ephemeral(&ctx).await;

    let data = ctx.data.read().await;
    let profile = data.get::<Profile>().unwrap();

    let (mut rows, row) = profile.categories.iter().enumerate().fold((vec![], vec![]), |acc, (idx, cat)| {
        let (mut rows, mut row) = acc;

        row.push(CreateButton::new(format!("create-channel-{}", idx))
            .label(cat.name.clone())
            .style(ButtonStyle::Secondary));

        if row.len() >= profile.per_row as usize {
            rows.push(CreateActionRow::Buttons(row));
            row = vec![];
        }

        (rows, row)
    });
    if !row.is_empty() {
        rows.push(CreateActionRow::Buttons(row));
    }

    let _ = cmd.channel_id.send_message(&ctx, CreateMessage::new()
        .add_embed(CreateEmbed::new()
            .title("You can only have a max of two channels across all categories!")
            .description("Press one of the buttons below to create a channel in the respective category"))
        .components(rows)).await;

    let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
        .content("Channel creation message sent!")).await;
}