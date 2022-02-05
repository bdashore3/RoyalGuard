use crate::commands::{
    autorole::*, bans::*, config::*, general::*, kicks::*, logging::*, member_info::*, mutes::*,
    new_members::*, purges::*, reaction_roles::*, support::*, warns::*,
};
use serenity::framework::standard::macros::group;

// All command groups
#[group]
#[help_available(false)]
#[commands(ping)]
pub struct General;

#[group("Bot Configuration")]
#[description = "Admin/Moderator commands that configure the bot"]
#[commands(prefix, moderator, mutechannel, genmuterole, resetprefix, restoredb)]
pub struct Config;

#[group("Generic Moderation")]
#[description = "All basic moderation commands"]
#[commands(
    ban,
    unban,
    warn,
    unwarn,
    warns,
    mute,
    unmute,
    mutes,
    kick,
    purge,
    reactionrole
)]
pub struct GenericMod;

#[group("Logging")]
#[description("Commands for logging activity")]
#[commands(log)]
pub struct Logging;

#[group("Information")]
#[description = "Various information commands"]
#[commands(get_member_info)]
pub struct Information;

#[group("New Members")]
#[description = "All commands for new member events"]
#[commands(welcome, autorole, leave)]
pub struct NewMembers;

#[group("Support")]
#[description = "Support and help commands"]
#[commands(help, support, info, privacy)]
pub struct Support;
