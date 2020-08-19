use serenity::framework::standard::macros::group;
use crate::commands::{
    general::*,
    config::*,
    bans::*
};

// All command groups
#[group]
#[help_available(false)]
#[commands(ping)]
pub struct General;

#[group("Bot Configuration")]
#[description = "Admin/Moderator commands that configure the bot"]
#[commands(prefix)]
pub struct Config;

#[group("Generic Moderation")]
#[description = "Bans, unbans, and kicks"]
#[commands(ban, unban)]
pub struct GenericMod;