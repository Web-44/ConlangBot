use crate::channel::check_channel_count;
use serenity::all::{ComponentInteraction, Context, CreateActionRow, CreateInputText, CreateInteractionResponse, CreateModal, InputTextStyle};
use serenity::builder::CreateInteractionResponseMessage;

pub async fn run(ctx: &Context, comp: ComponentInteraction) {
    let id = &comp.data.custom_id;
    match check_channel_count(comp.user.id, ctx).await {
        Ok(true) => {
            let _ = comp.create_response(&ctx, CreateInteractionResponse::Modal(
                CreateModal::new(id, "Create Channel")
                    .components(vec![
                        CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Short, "Name", "0")
                            .required(true)
                            .min_length(2)
                            .max_length(100)),
                        CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Paragraph, "Topic", "1")
                            .required(false)
                            .max_length(1024))
                    ])
            )).await;
        },
        Ok(false) => {
            let _ = comp.create_response(&ctx, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                .content("You can only have a max of two channels across all categories!")
                .ephemeral(true))).await;
        },
        Err(err) => {
            let _ = comp.create_response(&ctx, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                .content(format!("An error occurred: {err}"))
                .ephemeral(true))).await;
        }
    }
}

