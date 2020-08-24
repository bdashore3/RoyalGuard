use serenity::framework::standard::macros::group;
use crate::commands::{
    general::*,
    config::*,
    bans::*,
    warns::*,
    mutes::*
};

// All command groups
#[group]
#[help_available(false)]
#[commands(ping)]
pub struct General;

#[group("Bot Configuration")]
#[description = "Admin/Moderator commands that configure the bot"]
#[commands(prefix, moderator, mutechannel)]
pub struct Config;

#[group("Generic Moderation")]
#[description = "All basic moderation commands"]
#[commands(ban, unban, warn, unwarn, warns, mute, unmute)]
pub struct GenericMod;