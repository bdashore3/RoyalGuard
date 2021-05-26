use std::{
    borrow::Cow,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    helpers::{command_utils, embed_store, mute_helper::*, permissions_helper},
    ConnectionPool, MuteMap, PermissionType, RoyalError,
};
use serenity::{
    builder::CreateEmbed,
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::{
        channel::Message,
        id::{ChannelId, GuildId, RoleId, UserId},
    },
    prelude::Mentionable,
    utils::parse_channel,
};

#[command]
async fn mute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(());
    }

    if args.is_empty() {
        mute_help(ctx, msg.channel_id).await;

        return Ok(());
    }

    let mute_user = match args.single::<UserId>() {
        Ok(id) => Cow::Owned(id.to_user(ctx).await?),
        Err(_) => {
            if msg.mentions.is_empty() {
                msg.channel_id
                    .say(ctx, RoyalError::MissingError("user mention"))
                    .await?;

                return Ok(());
            }

            Cow::Borrowed(&msg.mentions[0])
        }
    };

    if mute_user.id == msg.author.id {
        msg.channel_id
            .say(ctx, RoyalError::SelfError("mute"))
            .await?;

        return Ok(());
    }

    if permissions_helper::check_moderator(ctx, msg, Some(mute_user.id)).await? {
        msg.channel_id
            .say(
                ctx,
                RoyalError::PermissionError(PermissionType::Mention(
                    "mute",
                    "administrator/moderator",
                )),
            )
            .await?;

        return Ok(());
    }

    let guild = msg.guild(ctx).await.unwrap();

    let mut member = match guild.member(ctx, mute_user.id).await {
        Ok(member) => member,
        Err(_) => {
            msg.channel_id
                .say(ctx, RoyalError::MissingError("user ID/mention"))
                .await?;

            return Ok(());
        }
    };

    let mute_info = handle_mute_role(ctx, &guild, Some(msg.channel_id)).await?;

    if member.roles.contains(&mute_info.mute_role_id) {
        msg.channel_id
            .say(ctx, format!("{} is already muted!", mute_user.id.mention()))
            .await?;

        return Ok(());
    }

    member.add_role(ctx, mute_info.mute_role_id).await?;

    #[allow(unused_assignments)]
    let mut mute_embed = CreateEmbed::default();

    if !args.is_empty() {
        let time_check = args.single::<String>().unwrap();
        let number_check = &time_check[time_check.len() - 1..];

        if matches!(number_check, "w" | "d" | "h" | "m" | "s") {
            let mute_time_num = match time_check[..time_check.len() - 1].parse::<u64>() {
                Ok(num) => command_utils::get_time(num, number_check)?,
                Err(_) => {
                    msg.channel_id
                        .say(ctx, RoyalError::MissingError("integer"))
                        .await?;

                    return Ok(());
                }
            };

            prepare_mute_timer(ctx, mute_user.id, guild.id, mute_time_num).await?;

            mute_embed = embed_store::get_mute_embed(&mute_user, true, true, Some(&time_check));
        } else {
            msg.channel_id.say(ctx, "Please enter the correct syntax for a timed mute! Check the help for more information").await?;

            return Ok(());
        }
    } else {
        mute_embed = embed_store::get_mute_embed(&mute_user, true, false, None);
    }

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.0 = mute_embed.0;
                e
            })
        })
        .await?;

    Ok(())
}

#[command]
async fn unmute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(());
    }

    if args.is_empty() {
        mute_help(ctx, msg.channel_id).await;

        return Ok(());
    }

    let mute_user = match args.single::<UserId>() {
        Ok(id) => Cow::Owned(id.to_user(ctx).await?),
        Err(_) => {
            if msg.mentions.is_empty() {
                msg.channel_id
                    .say(ctx, RoyalError::MissingError("user mention"))
                    .await?;

                return Ok(());
            }

            Cow::Borrowed(&msg.mentions[0])
        }
    };

    let guild = msg.guild(ctx).await.unwrap();

    let mut member = match guild.member(ctx, mute_user.id).await {
        Ok(member) => member,
        Err(_) => {
            msg.channel_id
                .say(ctx, RoyalError::MissingError("user ID/mention"))
                .await?;

            return Ok(());
        }
    };

    let mute_info = handle_mute_role(ctx, &guild, Some(msg.channel_id)).await?;

    if !member.roles.contains(&mute_info.mute_role_id) {
        msg.channel_id
            .say(ctx, format!("{} is not muted!", mute_user.id.mention()))
            .await?;

        return Ok(());
    }

    {
        let mute_map = ctx.data.read().await.get::<MuteMap>().cloned().unwrap();

        let wrapped_mute = mute_map.get(&(guild.id, mute_user.id));

        if let Some(mute_guard) = wrapped_mute {
            mute_guard.value().abort();
        }
    }

    member.remove_role(ctx, mute_info.mute_role_id).await?;

    let mute_embed = embed_store::get_mute_embed(&mute_user, false, false, None);
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.0 = mute_embed.0;
                e
            })
        })
        .await?;

    Ok(())
}

#[command]
async fn genmuterole(ctx: &Context, msg: &Message) -> CommandResult {
    if !permissions_helper::check_administrator(ctx, msg, None).await? {
        return Ok(());
    }

    let guild = msg.guild(ctx).await.unwrap();
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let mute_info = sqlx::query!(
        "SELECT muted_role_id, mute_channel_id FROM guild_info WHERE guild_id = $1",
        guild.id.0 as i64
    )
    .fetch_one(&pool)
    .await?;

    let mute_role_exists = match mute_info.muted_role_id {
        Some(mute_role_id) => {
            let fetched_mute_role_id = RoleId::from(mute_role_id as u64);

            guild.roles.contains_key(&fetched_mute_role_id)
        }
        None => false,
    };

    let new_mute_channel_id = match mute_info.mute_channel_id {
        Some(channel_id) => ChannelId::from(channel_id as u64),
        None => msg.channel_id,
    };

    if mute_role_exists {
        msg.channel_id
            .say(ctx, "This server already has a muted role! Aborting...")
            .await?;
    } else {
        new_mute_role(ctx, &guild, new_mute_channel_id).await?;

        let response = concat!(
            "Created a new role called `Muted`. \n",
            "Feel free to customize this role as much as you want \n",
            "If you accidentally delete this role, a new one will be created \n",
            "All channels have been updated with the mute role \n",
            "Use `mutechannel` to change where timed unmutes are sent"
        );

        msg.channel_id.say(ctx, response).await?;
    }

    Ok(())
}

#[command]
async fn mutechannel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_administrator(ctx, msg, None).await? {
        return Ok(());
    }

    let test_id = args
        .single::<String>()
        .unwrap_or_else(|_| msg.channel_id.mention().to_string());

    let channel_id = match parse_channel(&test_id) {
        Some(channel_id) => ChannelId::from(channel_id),
        None => {
            msg.channel_id
                .say(ctx, RoyalError::MissingError("mentioned channel"))
                .await?;

            return Ok(());
        }
    };

    let guild_id = msg.guild_id.unwrap();

    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    sqlx::query!(
        "UPDATE guild_info SET mute_channel_id = $1 WHERE guild_id = $2",
        channel_id.0 as i64,
        guild_id.0 as i64
    )
    .execute(&pool)
    .await?;

    let mute_channel_embed = embed_store::get_channel_embed(channel_id, "Mute");

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.0 = mute_channel_embed.0;
                e
            })
        })
        .await?;

    Ok(())
}

pub async fn mute_help(ctx: &Context, channel_id: ChannelId) {
    let content = concat!(
        "mute <mention> (time(w, d, h, m, s)): Mutes the mentioned user. Creates a role if it doesn't exist.",
            "If a time is provided, the user will be muted for a period of time \n\n",
        "unmute <mention>: Unmutes the mentioned user. Overrides all time based mutes \n\n",
        "mutechannel (channel Id): Sets the channel where timed unmutes are sent. This is where the mute role is created by default",
        "genmuterole: A command to manually generate the mute role. Also sets the mutechannel if one isn't already set");

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Mute help");
                e.description("Description: Commands for muting/silencing users in a server");
                e.field("Commands", content, false);
                e
            })
        })
        .await;
}
