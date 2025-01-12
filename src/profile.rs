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
    pub roles: ProfileRoles,
    pub categories: Vec<Category>
}

impl Profile {
    pub fn guild(&self) -> GuildId {
        GuildId::new(self.guild)
    }

    pub fn is_archive(&self, id: ChannelId) -> bool {
        self.archives.contains(&id.get()) || self.private_archives.contains(&id.get())
    }
}

#[derive(Clone, Deserialize)]
pub struct ProfileRoles {
    pub everyone: u64,
    pub member: u64,
    pub conlanger: u64
}

impl ProfileRoles {
    pub fn everyone(&self) -> RoleId {
        RoleId::new(self.everyone)
    }

    pub fn member(&self) -> RoleId {
        RoleId::new(self.member)
    }

    pub fn conlanger(&self) -> RoleId {
        RoleId::new(self.conlanger)
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