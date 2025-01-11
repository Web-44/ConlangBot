use std::path::Path;
use std::sync::Arc;
use crate::commands::DEVELOPER;
use serenity::all::{ChannelId, CommandDataOptionValue, CommandInteraction, CommandOptionType, Context, EditInteractionResponse, UserId};
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::Permissions;
use tokio::fs;
use tokio::time::Instant;
use crate::channel::ConChannel;
use crate::database::{add_channel, SqlPool};
use crate::DatabasePoolKey;

pub fn register() -> CreateCommand {
    CreateCommand::new("migrate")
        .description("Migrate data from an old database")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .dm_permission(true)
        .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "funky_text", "Text format by Funky")
            .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "path", "Path to the channels folder")
                .required(true)))
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let _ = cmd.defer(&ctx).await;

    if cmd.user.id != DEVELOPER {
        let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
            .content("You are not allowed to use this command")).await;
        return;
    }

    let data = ctx.data.read().await;
    let database_pool = data.get::<DatabasePoolKey>().unwrap();

    if let CommandDataOptionValue::SubCommand(options) = &cmd.data.options[0].value {
        let time = Instant::now();

        let result = match cmd.data.options[0].name.as_str() {
            "txt" => {
                if let Some(path) = options[0].value.as_str() {
                    migrate_funky_text(database_pool, path).await
                } else {
                    Err("Path not provided".to_string())
                }
            }
            _ => Err("Type not implemented".to_string())
        };

        let time = time.elapsed().as_millis();

        if let Err(err) = result {
            let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                .content(format!("Migration failed after {time} ms: {err}"))).await;
        } else {
            let _ = cmd.edit_response(&ctx, EditInteractionResponse::new()
                .content(format!("Migration successful (took {time} ms), check console for warnings"))).await;
        }
    }
}

async fn migrate_funky_text(database_pool: &Arc<SqlPool>, path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if !path.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    if let Ok(mut dir) = fs::read_dir(path).await {
        while let Some(entry) = dir.next_entry().await.map_err(|e| e.to_string())? {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name() {
                    if let Some(name) = name.to_str() {
                        if let Some(channel_id) = name.strip_suffix(".txt") {
                            if let Ok(channel_id) = channel_id.parse::<u64>() {
                                if let Ok(content) = fs::read_to_string(&path).await {
                                    let (owner, category) = content.split("\n").fold((None, None), |acc, line| {
                                        let (mut owner, mut category) = acc;

                                        if let Some((name, value)) = line.split_once(":") {
                                            match name {
                                                "A" => {
                                                    if let Ok(owner_id) = value.parse::<u64>() {
                                                        owner = Some(owner_id);
                                                    }
                                                }
                                                "C" => {
                                                    if let Ok(category_id) = value.parse::<u64>() {
                                                        category = Some(category_id);
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }

                                        (owner, category)
                                    });

                                    if let (Some(owner), Some(category)) = (owner, category) {
                                        add_channel(database_pool.clone(), ConChannel {
                                            id: ChannelId::new(channel_id),
                                            owner: UserId::new(owner),
                                            category
                                        }).await.map_err(|err| format!("Failed to add {path:?} ({channel_id}, {owner}, {category}): {err:?}"))?;
                                    } else {
                                        eprintln!("Failed to read owner and category from {path:?}");
                                    }
                                } else {
                                    eprintln!("Failed to read file {path:?}");
                                }
                            } else {
                                eprintln!("Failed to parse channel id from {channel_id}");
                            }
                        } else {
                            eprintln!("{name} does not fit .txt name format");
                        }
                    } else {
                        eprintln!("Failed to convert file name to string: {name:?}");
                    }
                } else {
                    eprintln!("Failed to get file name of {path:?}");
                }
            } else {
                eprintln!("Skipping non-file: {path:?}");
            }
        }
        Ok(())
    } else {
        Err("Failed to read directory".to_string())
    }
}