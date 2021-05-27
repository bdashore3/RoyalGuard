use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, NaiveDateTime, Utc};
use futures::future::{AbortHandle, Abortable};
use itertools::Itertools;
use serenity::{
    client::Context,
    framework::standard::CommandResult,
    model::{
        channel::{ChannelType, PermissionOverwrite, PermissionOverwriteType},
        guild::Guild,
        id::{ChannelId, GuildId, RoleId, UserId},
        Permissions,
    },
    prelude::Mentionable,
};
use sqlx::PgPool;
use tokio::time::sleep;

use crate::{helpers::embed_store, ConnectionPool, MuteMap};

#[derive(Default, Debug)]
pub struct MuteInfo {
    pub mute_role_id: RoleId,
    pub mute_channel_id: ChannelId,
}

pub async fn prepare_mute_timer(
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

pub async fn create_mute_timer(ctx: Context, time: u64, user_id: UserId, guild_id: GuildId) {
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

pub async fn unmute_by_time(ctx: &Context, user_id: &UserId, guild_id: &GuildId) -> CommandResult {
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
        Err(_) => return Ok(()),
    };

    let mute_info = handle_mute_role(ctx, &guild, None, false).await?;

    if !member.roles.contains(&mute_info.mute_role_id) {
        return Ok(());
    }

    match member.remove_role(ctx, mute_info.mute_role_id).await {
        Ok(()) => {
            let mute_embed =
                embed_store::get_mute_embed(&user_id.to_user(ctx).await?, false, false, None);

            mute_info
                .mute_channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.0 = mute_embed.0;
                        e
                    })
                })
                .await?;
        }
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

pub async fn handle_mute_role(
    ctx: &Context,
    guild: &Guild,
    channel_id: Option<ChannelId>,
    forced_gen: bool,
) -> CommandResult<MuteInfo> {
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
        if forced_gen {
            channel_id
                .say(ctx, "This server already has a muted role! Aborting...")
                .await?;
        }

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

pub async fn new_mute_role(
    ctx: &Context,
    guild: &Guild,
    channel_id: ChannelId,
) -> CommandResult<MuteInfo> {
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

pub async fn fetch_guild_mutes(
    pool: &PgPool,
    guild: &Guild,
    mute_role_id: RoleId,
) -> CommandResult<(String, String)> {
    let timed_mutes = fetch_timed_mutes(pool, &guild.id).await?;

    let permanent_mute_test = guild
        .members
        .iter()
        .filter(|(u, m)| m.roles.contains(&mute_role_id) && !timed_mutes.contains_key(&u))
        .format_with(" \n", |(u, _), f| f(&u.mention()))
        .to_string();

    let permanent_mute_string = if permanent_mute_test.is_empty() {
        "No permanent mutes!".to_string()
    } else {
        permanent_mute_test
    };

    let timed_mute_string = if timed_mutes.is_empty() {
        "No temporary mutes!".to_string()
    } else {
        timed_mutes
            .iter()
            .format_with(" \n", |(user_id, timestamp), f| {
                f(&format_args!("{}: {}", user_id.mention(), timestamp))
            })
            .to_string()
    };

    Ok((permanent_mute_string, timed_mute_string))
}

pub async fn fetch_timed_mutes(
    pool: &PgPool,
    guild_id: &GuildId,
) -> CommandResult<HashMap<UserId, String>> {
    let mute_data_vec = sqlx::query!("SELECT * FROM mutes WHERE guild_id = $1", guild_id.0 as i64)
        .fetch_all(pool)
        .await?;

    let timed_mutes = mute_data_vec
        .iter()
        .map(|mute_data| {
            let naive_datetime = NaiveDateTime::from_timestamp(mute_data.mute_time, 0);
            let datetime_again: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);

            let user_id = UserId(mute_data.user_id as u64);

            (user_id, datetime_again.to_string())
        })
        .collect::<HashMap<UserId, String>>();

    Ok(timed_mutes)
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

        println!("\n");
        println!("UserID: {}", &i.user_id);
        println!("GuildID: {}", &i.guild_id);
        println!("Time difference: {}", mute_time_diff);
        println!("\n");

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
