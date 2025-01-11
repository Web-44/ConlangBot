use serde::Deserialize;
use serenity::all::{ChannelId, GuildId, RoleId};
use serenity::prelude::TypeMapKey;
use std::fs::File;
use std::io::BufReader;

#[derive(Clone, Deserialize)]
pub struct Profile {
    pub name: String,
    pub guild: u64,
    pub archives: Vec<u64>,
    #[serde(rename = "private-archives")]
    pub private_archives: Vec<u64>,
    #[serde(rename = "per-row")]
    pub per_row: u8,
    #[serde(rename = "everyone-role")]
    pub everyone_role: u64,
    pub categories: Vec<Category>
}

impl Profile {
    pub fn guild(&self) -> GuildId {
        GuildId::new(self.guild)
    }

    pub fn is_archive(&self, id: ChannelId) -> bool {
        self.archives.contains(&id.get()) || self.private_archives.contains(&id.get())
    }

    pub fn everyone(&self) -> RoleId {
        RoleId::new(self.everyone_role)
    }
}

#[derive(Clone, Deserialize)]
pub struct Category {
    pub id: u64,
    pub name: String
}

impl TypeMapKey for Profile {
    type Value = Profile;
}

pub fn read_profile(path: String) -> Profile {
    let file = File::open(path).expect("Failed to open profile file");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).expect("Failed to parse profile file")
}