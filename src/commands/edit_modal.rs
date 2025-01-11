use crate::channel::sort_category;
use crate::database::get_channel_by_id;
use crate::DatabasePoolKey;
use serenity::all::{ActionRowComponent, ModalInteraction};
use serenity::builder::{EditChannel, EditInteractionResponse};
use serenity::client::Context;

pub async fn run(ctx: &Context, modal: ModalInteraction) {
    let _ = modal.defer_ephemeral(&ctx).await;

    let data = ctx.data.read().await;
    let database_pool = data.get::<DatabasePoolKey>().unwrap();

    if let Ok(channel) = get_channel_by_id(database_pool.clone(), modal.channel_id).await {
        if channel.owner == modal.user.id {
            if modal.data.components.len() == 2 &&
                !modal.data.components[0].components.is_empty() &&
                !modal.data.components[1].components.is_empty() {
                if let ActionRowComponent::InputText(txt) = &modal.data.components[0].components[0] {
                    if let Some(channel_name) = &txt.value {
                        if let ActionRowComponent::InputText(txt) = &modal.data.components[1].components[0] {
                            if let Some(channel_topic) = &txt.value {
                                match modal.channel_id.edit(&ctx, EditChannel::new()
                                    .name(channel_name)
                                    .topic(channel_topic))
                                    .await {
                                    Ok(guild_channel) => {
                                        if let Some(parent_id) = guild_channel.parent_id {
                                            let _ = sort_category(parent_id.get(), &ctx).await;
                                        }
                                        let _ = modal.edit_response(&ctx, EditInteractionResponse::new()
                                            .content("Channel edited!")).await;
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
        } else {
            let _ = modal.edit_response(&ctx, EditInteractionResponse::new()
                .content("You can only edit your own channel")).await;
        }
    } else {
        let _ = modal.edit_response(&ctx, EditInteractionResponse::new()
            .content("Could not find channel data")).await;
    }
}