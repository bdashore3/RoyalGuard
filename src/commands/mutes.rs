use crate::{
    helpers::{command_utils, embed_store, permissions_helper},
    ConnectionPool, MuteMap, PermissionType, RoyalError,
};
use futures::future::{AbortHandle, Abortable};
use serenity::{
    builder::CreateEmbed,
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::parse_channel,
};
use std::{
    borrow::Cow,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::sleep;

#[derive(Default, Debug)]
struct MuteInfo {
    mute_role_id: RoleId,
    mute_channel_id: ChannelId,
}

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

async fn prepare_mute_timer(
    ctx: &Context,
    user_id: UserId,
    guild_id: GuildId,
    mute_time_num: u64,
) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards?");

    let future_time = current_time.as_secs() + mute_time_num;

    sqlx::query!(
        "INSERT INTO mutes
            VALUES($1, $2, $3)
            ON CONFLICT (guild_id, user_id)
            DO UPDATE
            SET mute_time = EXCLUDED.mute_time",
        guild_id.0 as i64,
        user_id.0 as i64,
        future_time as i64
    )
    .execute(&pool)
    .await?;

    let ctx_clone = ctx.clone();

    tokio::spawn(async move {
        create_mute_timer(ctx_clone, mute_time_num, user_id, guild_id).await;
    });

    Ok(())
}

async fn create_mute_timer(ctx: Context, time: u64, user_id: UserId, guild_id: GuildId) {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let future = Abortable::new(
        unmute_by_time(&ctx, &user_id, &guild_id),
        abort_registration,
    );

    let mute_map = ctx.data.read().await.get::<MuteMap>().cloned().unwrap();
    mute_map.insert((guild_id, user_id), abort_handle);

    sleep(Duration::from_secs(time)).await;
    match future.await {
        Ok(_) => {}
        Err(_e) => {
            let pool = ctx
                .data
                .read()
                .await
                .get::<ConnectionPool>()
                .cloned()
                .unwrap();

            if let Err(e) = sqlx::query!(
                "DELETE FROM mutes WHERE guild_id = $1 AND user_id = $2",
                guild_id.0 as i64,
                user_id.0 as i64
            )
            .execute(&pool)
            .await
            {
                eprintln!(
                    "Error when deleting mute entry from user {} in guild {}: {}",
                    user_id.0, guild_id.0, e
                );
            }
        }
    }
}

async fn unmute_by_time(ctx: &Context, user_id: &UserId, guild_id: &GuildId) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let guild = match ctx.cache.guild(guild_id).await {
        Some(guild) => guild,
        None => {
            eprintln!(
                "There was an error in finding guild {} from the cache!",
                guild_id.0
            );

            return Ok(());
        }
    };

    sqlx::query!(
        "DELETE FROM mutes WHERE guild_id = $1 AND user_id = $2",
        guild.id.0 as i64,
        user_id.0 as i64
    )
    .execute(&pool)
    .await?;

    let mut member = match guild.member(ctx, user_id).await {
        Ok(member) => member,
        Err(_) => {
            return Ok(())
        }
    };

    let mute_info = handle_mute_role(ctx, &guild, None).await?;

    if !member.roles.contains(&mute_info.mute_role_id) {
        return Ok(());
    }

    match member.remove_role(ctx, mute_info.mute_role_id).await {
        Ok(()) => {
            let mute_embed = embed_store::get_mute_embed(&user_id.to_user(ctx).await?, false, false, None);

            mute_info
                .mute_channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.0 = mute_embed.0;
                        e
                    })
                })
                .await?;
        },
        Err(_) => {
            mute_info
                .mute_channel_id
                .say(
                    ctx, 
                    format!(
                        "I could not remove the mute role from user {} with ID: {}. Please manually remove the `muted` role.",
                        user_id.mention(),
                        user_id
                    )
                ).await?;
        }
    };

    Ok(())
}

async fn handle_mute_role(
    ctx: &Context,
    guild: &Guild,
    channel_id: Option<ChannelId>,
) -> Result<MuteInfo, Box<dyn std::error::Error + Send + Sync>> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let mute_data = sqlx::query!(
        "SELECT muted_role_id, mute_channel_id FROM guild_info WHERE guild_id = $1",
        guild.id.0 as i64
    )
    .fetch_one(&pool)
    .await?;

    let channel_id = match channel_id {
        Some(id) => id,
        None => ChannelId::from(mute_data.mute_channel_id.unwrap() as u64),
    };

    if mute_data.muted_role_id.is_none() {
        let new_mute_string = concat!(
            "Created a new role called `Muted`. \n",
            "Feel free to customize this role as much as you want \n",
            "If you accidentally delete this role, a new one will be created \n",
            "All channels have been updated with the mute role \n",
            "Use `mutechannel` to change where timed unmutes are sent"
        );

        channel_id.say(ctx, new_mute_string).await?;

        let mute_info = new_mute_role(ctx, guild, channel_id).await?;

        return Ok(mute_info);
    }

    let mute_role_id = RoleId::from(mute_data.muted_role_id.unwrap() as u64);

    if guild.roles.contains_key(&mute_role_id) {
        let mute_info = MuteInfo {
            mute_role_id,
            mute_channel_id: channel_id,
        };

        return Ok(mute_info);
    } else {
        channel_id.say(ctx,
            "You deleted the mute role from your server, but the database wasn't updated! Recreating role...").await?;

        let mute_info = new_mute_role(ctx, guild, channel_id).await?;

        return Ok(mute_info);
    }
}

async fn new_mute_role(
    ctx: &Context,
    guild: &Guild,
    channel_id: ChannelId,
) -> Result<MuteInfo, Box<dyn std::error::Error + Send + Sync>> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let mute_role = guild
        .create_role(ctx, |r| {
            r.name("muted");
            r.permissions(Permissions::READ_MESSAGES | Permissions::READ_MESSAGE_HISTORY);
            r
        })
        .await?;

    let overwrite_voice = PermissionOverwrite {
        allow: Permissions::empty(),
        deny: Permissions::CONNECT | Permissions::SPEAK | Permissions::STREAM,
        kind: PermissionOverwriteType::Role(mute_role.id),
    };

    let overwrite_text = PermissionOverwrite {
        allow: Permissions::empty(),
        deny: Permissions::SEND_MESSAGES | Permissions::SEND_TTS_MESSAGES,
        kind: PermissionOverwriteType::Role(mute_role.id),
    };

    for channel in &guild.channels {
        if channel.1.kind == ChannelType::Voice {
            channel.1.create_permission(ctx, &overwrite_voice).await?;
        } else {
            channel.1.create_permission(ctx, &overwrite_text).await?;
        }
    }

    sqlx::query!(
        "UPDATE guild_info SET muted_role_id = $1, mute_channel_id = $2 WHERE guild_id = $3",
        mute_role.id.0 as i64,
        channel_id.0 as i64,
        guild.id.0 as i64
    )
    .execute(&pool)
    .await?;

    let mute_info = MuteInfo {
        mute_role_id: mute_role.id,
        mute_channel_id: channel_id,
    };

    Ok(mute_info)
}

pub async fn load_mute_timers(ctx: &Context) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let timer_data = sqlx::query!("SELECT guild_id, user_id, mute_time FROM mutes")
        .fetch_all(&pool)
        .await?;

    for i in timer_data {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards?")
            .as_secs() as i64;

        let mute_time_diff = i.mute_time - current_time;

        println!("UserID: {}", &i.user_id);
        println!("GuildID: {}", &i.guild_id);
        println!("Time difference: {}", mute_time_diff);

        let guild_id = GuildId::from(i.guild_id as u64);
        let user_id = UserId::from(i.user_id as u64);

        if mute_time_diff <= 0 {
            let check = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM delete_time_store WHERE guild_id = $1)",
                i.guild_id
            )
            .fetch_one(&pool)
            .await?;

            if !check.exists.unwrap() {
                println!("Unmuting user: {}", user_id.0);

                unmute_by_time(&ctx, &user_id, &guild_id).await?;
            }
        } else {
            let ctx_clone = ctx.clone();

            tokio::spawn(async move {
                create_mute_timer(ctx_clone, mute_time_diff as u64, user_id, guild_id).await;
            });
        }
    }

    Ok(())
}

pub async fn mute_help(ctx: &Context, channel_id: ChannelId) {
    let content = concat!(
        "mute <mention> (time(w, d, h, m, s)): Mutes the mentioned user. Creates a role if it doesn't exist.",
            "If a time is provided, the user will be muted for a period of time \n\n",
        "unmute <mention>: Unmutes the mentioned user. Overrides all time based mutes \n\n",
        "mutechannel (channel Id): Sets the channel where timed unmutes are sent. This is where the mute role is created by default");

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
