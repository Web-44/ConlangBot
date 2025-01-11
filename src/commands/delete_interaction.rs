use crate::database::get_channel_by_id;
use crate::DatabasePoolKey;
use serenity::all::{ComponentInteraction, Context};
use serenity::builder::EditInteractionResponse;

pub async fn run(ctx: &Context, comp: ComponentInteraction) {
    let _ = comp.defer_ephemeral(&ctx).await;

    let data = ctx.data.read().await;
    let database_pool = data.get::<DatabasePoolKey>().unwrap();

    if let Ok(channel) = get_channel_by_id(database_pool.clone(), comp.channel_id).await {
        if channel.owner == comp.user.id {
            let _ = comp.edit_response(&ctx, EditInteractionResponse::new()
                .content("Deleting channel…")).await;
            let _ = comp.channel_id.delete(&ctx).await;
        } else {
            let _ = comp.edit_response(&ctx, EditInteractionResponse::new()
                .content("You can only delete your own channel")).await;
        }
    } else {
        let _ = comp.edit_response(&ctx, EditInteractionResponse::new()
            .content("Could not find channel data")).await;
    }
}

