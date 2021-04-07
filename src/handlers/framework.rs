use std::collections::HashSet;

use crate::{
    helpers::command_utils,
    structures::{
        cmd_data::{EmergencyCommands, PrefixMap, PubCreds},
        commands::*,
        errors::*,
    },
};
use serenity::{
    client::Context,
    framework::standard::{macros::hook, CommandError, DispatchError, StandardFramework},
    model::{channel::Message, id::UserId, Permissions},
    prelude::Mentionable,
};

pub async fn get_framework(bot_id: UserId, owners: HashSet<UserId>) -> StandardFramework {
    StandardFramework::new()
        .configure(|c| {
            c.dynamic_prefix(dynamic_prefix)
                .prefix("")
                .on_mention(Some(bot_id))
                .owners(owners)
        })
        .on_dispatch_error(dispatch_error)
        .before(before)
        .after(after)
        .group(&GENERAL_GROUP)
        .group(&CONFIG_GROUP)
        .group(&GENERICMOD_GROUP)
        .group(&NEWMEMBERS_GROUP)
        .group(&INFORMATION_GROUP)
        .group(&SUPPORT_GROUP)
}

#[hook]
async fn before(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    if command_utils::check_mention_prefix(msg) {
        let emergency_commands = ctx
            .data
            .read()
            .await
            .get::<EmergencyCommands>()
            .cloned()
            .unwrap();

        if emergency_commands.contains(&cmd_name.to_owned()) {
            let _ = msg
                .channel_id
                .say(
                    ctx,
                    format!(
                        "{}, you are running an emergency command!",
                        msg.author.mention()
                    ),
                )
                .await;
            return true;
        } else {
            return false;
        }
    }

    true
}

// After a command is executed, go here
#[hook]
async fn after(ctx: &Context, msg: &Message, cmd_name: &str, error: Result<(), CommandError>) {
    if let Err(why) = error {
        let part_1 = "Looks like the bot encountered an error! \n";
        let part_2 = "Please use the `support` command and send the output to the support server!";
        let error_string = format!("{}{}", part_1, part_2);

        let _ = msg
            .channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.color(0xff69b4);
                    e.title("Aw Snap!");
                    e.description(error_string);
                    e.field("Command Name", cmd_name, false);
                    e.field("Error", format!("```{} \n```", why), false);
                    e
                })
            })
            .await;
    }
}

// On a dispatch error, go here. Catches most permission errors.
#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    match error {
        DispatchError::LackingPermissions(Permissions::ADMINISTRATOR) => {
            let _ = msg
                .channel_id
                .say(
                    ctx,
                    RoyalError::PermissionError(PermissionType::SelfPerm("administrator")),
                )
                .await;
        }
        DispatchError::NotEnoughArguments { min, given } => {
            let _ = msg
                .channel_id
                .say(
                    ctx,
                    format!("Args required: {}. Args given: {}", min, given),
                )
                .await;
        }
        DispatchError::OnlyForOwners => {
            let _ = msg
                .channel_id
                .say(ctx, "This is a bot dev only command!")
                .await;
        }
        _ => println!("Unhandled dispatch error: {:?}", error),
    }
}

#[hook]
async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
    let (prefixes, default_prefix) = {
        let data = ctx.data.read().await;
        let prefixes = data.get::<PrefixMap>().cloned().unwrap();
        let default_prefix = data
            .get::<PubCreds>()
            .unwrap()
            .get("default prefix")
            .cloned()
            .unwrap();

        (prefixes, default_prefix)
    };

    let guild_id = msg.guild_id.unwrap();

    let wrapped_prefix = prefixes.get(&guild_id);

    match wrapped_prefix {
        Some(prefix_guard) => Some(prefix_guard.value().to_owned()),
        None => Some(default_prefix),
    }
}
