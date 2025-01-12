use crate::channel::create_channel;
use serenity::all::{ActionRowComponent, Context, ModalInteraction};
use serenity::builder::EditInteractionResponse;

pub async fn run(ctx: &Context, modal: ModalInteraction) {
    let _ = modal.defer_ephemeral(&ctx).await;
    if let Ok(idx) = modal.data.custom_id[15..].parse::<u8>() {
        let data = ctx.data.read().await;
        let profile = data.get::<crate::profile::Profile>().unwrap();
        let cat = &profile.categories[idx as usize];

        if modal.data.components.len() == 2 &&
            !modal.data.components[0].components.is_empty() &&
            !modal.data.components[1].components.is_empty() {
            if let ActionRowComponent::InputText(txt) = &modal.data.components[0].components[0] {
                if let Some(channel_name) = &txt.value {
                    if let ActionRowComponent::InputText(txt) = &modal.data.components[1].components[0] {
                        if let Some(channel_topic) = &txt.value {
                            match create_channel(modal.user.id, channel_name, channel_topic, cat, &ctx).await {
                                Ok(channel) => {
                                    if let Some(member) = &modal.member {
                                        let _ = member.add_role(&ctx, profile.roles.conlanger()).await;
                                    }
                                    let _ = modal.edit_response(&ctx, EditInteractionResponse::new()
                                        .content(format!("Channel created: <#{}>", channel.id))).await;
                                }
                                Err(err) => {
                                    let _ = modal.edit_response(&ctx, EditInteractionResponse::new().content(format!("Error: {err}"))).await;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}