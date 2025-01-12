use crate::database::{add_channel, get_channels_by_owner};
use crate::profile::{Category, Profile};
use crate::DatabasePoolKey;
use serenity::all::{ChannelId, ChannelType, CreateChannel, GuildChannel, Member, PermissionOverwrite, PermissionOverwriteType, User, UserId};
use serenity::client::Context;
use serenity::model::Permissions;
use sqlx::Error;

#[macro_export]
macro_rules! perm_viewable {
    () => { Permissions::VIEW_CHANNEL };
}

#[macro_export]
macro_rules! perm_writable {
    () => { Permissions::SEND_MESSAGES | Permissions::SEND_MESSAGES_IN_THREADS | Permissions::ADD_REACTIONS };
}

#[macro_export]
macro_rules! owner {
    () => { perm_viewable!() | perm_writable!() | Permissions::CREATE_PUBLIC_THREADS | Permissions::CREATE_PRIVATE_THREADS |
        Permissions::MANAGE_THREADS | Permissions::MANAGE_MESSAGES | Permissions::MANAGE_CHANNELS };
}

#[macro_export]
macro_rules! hide_to_everyone {
    ($role: expr) => {
        PermissionOverwrite {
            allow: Permissions::empty(),
            deny: perm_viewable!(),
            kind: PermissionOverwriteType::Role($role)
        }
    };
}

#[macro_export]
macro_rules! channel_public {
    ($role: expr) => {
        PermissionOverwrite {
            allow: perm_viewable!() | perm_writable!(),
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role($role)
        }
    };
}

#[macro_export]
macro_rules! channel_viewable {
    ($role: expr) => {
        PermissionOverwrite {
            allow: perm_viewable!(),
            deny: perm_writable!(),
            kind: PermissionOverwriteType::Role($role)
        }
    };
}

#[macro_export]
macro_rules! user_owner {
    ($user: expr) => {
        PermissionOverwrite {
            allow: owner!(),
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Member($user)
        }
    };
}

#[macro_export]
macro_rules! user_collaborator {
    ($user: expr) => {
        PermissionOverwrite {
            allow: perm_viewable!() | perm_writable!(),
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Member($user)
        }
    };
}

#[macro_export]
macro_rules! user_viewer {
    ($user: expr) => {
        PermissionOverwrite {
            allow: perm_viewable!(),
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Member($user)
        }
    };
}

#[macro_export]
macro_rules! user_banned {
    ($user: expr) => {
        PermissionOverwrite {
            allow: Permissions::empty(),
            deny: perm_viewable!() | perm_writable!(),
            kind: PermissionOverwriteType::Member($user)
        }
    };
}

pub struct ConChannel {
    pub id: ChannelId,
    pub owner: UserId,
    pub category: Option<u64>
}

impl ConChannel {
    pub fn category(&self) -> Option<ChannelId> {
        self.category.map(ChannelId::new)
    }

    pub fn check_permission(&self, user: &User, invoker: &Option<Box<Member>>) -> bool {
        if self.owner == user.id {
            return true;
        }
        if let Some(invoker) = invoker {
            if let Some(permissions) = invoker.permissions {
                return permissions.contains(Permissions::MANAGE_CHANNELS);
            }
        }
        false
    }

    pub fn check_permission_unboxed(&self, user: &User, invoker: &Option<Member>) -> bool {
        if self.owner == user.id {
            return true;
        }
        if let Some(invoker) = invoker {
            if let Some(permissions) = invoker.permissions {
                return permissions.contains(Permissions::MANAGE_CHANNELS);
            }
        }
        false
    }
}

pub async fn check_channel_count(user: UserId, ctx: &Context) -> Result<bool, Error> {
    let data = ctx.data.read().await;
    let database_pool = data.get::<DatabasePoolKey>().unwrap();

    match get_channels_by_owner(database_pool.clone(), user).await {
        Ok(channels) => Ok(channels.len() < 2),
        Err(err) => {
            eprintln!("Error checking channel count of user {user}: {err:?}");
            Err(err)
        }
    }
}

pub async fn get_channels_in_category(category: u64, ctx: &Context) -> Result<Vec<GuildChannel>, String> {
    let data = ctx.data.read().await;
    let profile = data.get::<Profile>().unwrap();
    let guild = profile.guild();
    let category = ChannelId::new(category);
    let channels = guild.channels(&ctx).await.map_err(|err| format!("Failed to get channels: {err}"))?;
    let channels: Vec<GuildChannel> = channels.into_iter().filter_map(|(_, channel)| {
        if channel.parent_id == Some(category) {
            Some(channel)
        } else {
            None
        }
    }).collect();
    Ok(channels)
}

pub async fn sort_category(category: u64, ctx: &Context) -> Result<(), String> {
    let data = ctx.data.read().await;
    let profile = data.get::<Profile>().unwrap();
    let guild = profile.guild();
    let mut channels = get_channels_in_category(category, ctx).await?;
    channels.sort_by(|a, b| a.name.cmp(&b.name));
    guild.reorder_channels(&ctx, channels.into_iter().enumerate().map(|(idx, ch)| {
        (ch.id, idx as u64)
    })).await.map_err(|err| format!("Failed to reorder channels: {err}"))?;

    Ok(())
}

pub async fn create_channel(creator: UserId, channel_name: &str, channel_topic: &str, category: &Category, ctx: &Context) -> Result<GuildChannel, String> {
    let data = ctx.data.read().await;
    let profile = data.get::<Profile>().unwrap();
    let database_pool = data.get::<DatabasePoolKey>().unwrap();

    match check_channel_count(creator, ctx).await {
        Ok(true) => {},
        Ok(false) => return Err("You can only have a max of two channels across all categories!".to_string()),
        Err(err) => return Err(format!("An error occurred: {err}"))
    }

    let res = profile.guild().create_channel(&ctx, CreateChannel::new(channel_name)
        .topic(channel_topic)
        .kind(ChannelType::Text)
        .category(category.id)
        .permissions(vec![
            hide_to_everyone!(profile.roles.everyone()),
            user_owner!(creator),
            channel_public!(profile.roles.member())
        ])).await;

    match res {
        Ok(channel) => {
            match add_channel(database_pool.clone(), ConChannel {
                id: channel.id,
                owner: creator,
                category: Some(category.id)
            }).await {
                Ok(_) => {
                    let _ = sort_category(category.id, ctx).await;
                    Ok(channel)
                },
                Err(err) => {
                    let _ = channel.delete(&ctx).await;
                    eprintln!("Error adding channel #{channel_name} to database: {err:?}");
                    Err(err.to_string())
                }
            }
        },
        Err(err) => {
            eprintln!("Error creating channel #{channel_name}: {err:?}");
            Err(format!("An error occurred: {err}"))
        }
    }
}