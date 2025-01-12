use serenity::all::UserId;

pub mod archive;
pub mod ban;
pub mod category;
pub mod contributor;
pub mod create;
pub mod create_interaction;
pub mod create_modal;
pub mod debug;
pub mod delete;
pub mod delete_interaction;
pub mod edit;
pub mod edit_modal;
pub mod fixperms;
pub mod migrate;
pub mod mode;
pub mod unban;
pub mod viewer;
pub mod wordgen;

pub const DEVELOPER: UserId = UserId::new(796368453152800778);