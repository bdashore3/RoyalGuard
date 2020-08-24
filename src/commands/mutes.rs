use serenity::{
    prelude::*,
    model::prelude::*,
    builder::CreateEmbed, 
    framework::standard::{
        CommandResult,
        macros::command, 
        Args
    }, 
    utils::parse_channel
};
use std::time::{Duration, UNIX_EPOCH, SystemTime};
use crate::{
    ConnectionPool, 
    MuteMap, 
    helpers::{
        embed_store,
        permissions_helper,
        time_conversion
    }
};
use futures::future::{Abortable, AbortHandle};
use tokio::time::delay_for;

#[derive(Default, Debug)]
struct MuteInfo {
    mute_role_id: RoleId,
    mute_channel_id: ChannelId
}

#[command]
async fn mute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        msg.channel_id.say(ctx, "You can't execute this command because you're not a moderator on this server!").await?;

        return Ok(())
    }
    
    if msg.mentions.len() < 1 {
        msg.channel_id.say(ctx, "Please mention the user you want to mute!").await?;

        return Ok(())
    }

    let user_id = msg.mentions[0].id;

    if user_id == msg.author.id {
        msg.channel_id.say(ctx, "I don't think you can mute yourself.").await?;

        return Ok(())
    }

    if permissions_helper::check_moderator(ctx, msg, Some(user_id)).await? {
        msg.channel_id.say(ctx, "I can't mute an administrator/moderator! Please demote the user then try again.").await?;

        return Ok(())
    }

    let guild = msg.guild(ctx).await.unwrap();
    let mut member = guild.member(ctx, user_id).await?;

    let mute_info = handle_mute_role(ctx, &guild, Some(msg.channel_id)).await?;

    if member.roles.contains(&mute_info.mute_role_id) {
        msg.channel_id.say(ctx, format!("{} is already muted!", user_id.mention())).await?;

        return Ok(())
    }

    member.add_role(ctx, mute_info.mute_role_id).await?;

    #[allow(unused_assignments)]
    let mut mute_embed = CreateEmbed::default();

    args.advance();

    if !args.is_empty() {
        let time_check  = args.single::<String>().unwrap();
        let number_check = &time_check[time_check.len() - 1..];
    
        if matches!(number_check, "w"|"d"|"h"|"m"|"s") {
            let mute_time_num = match time_check[..time_check.len() - 1].parse::<u64>() {
                Ok(num) => time_conversion::get_time(num, number_check)?,
                Err(_) => {
                    msg.channel_id.say(ctx, "Please provide an integer!").await?;
    
                    return Ok(())
                }
            };

            prepare_mute_timer(ctx, user_id, guild.id, mute_time_num).await?;

            mute_embed = embed_store::get_mute_embed(&msg.mentions[0], true, true, Some(&time_check));
        } else {
            msg.channel_id.say(ctx, "Please enter the correct syntax for a timed mute! Check the help for more information").await?;

            return Ok(())
        }
    } else {
        mute_embed = embed_store::get_mute_embed(&msg.mentions[0], true, false, None);
    }

    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.0 = mute_embed.0;
            e
        })
    }).await?;

    Ok(())
}

#[command]
async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        msg.channel_id.say(ctx, "You can't execute this command because you're not a moderator on this server!").await?;

        return Ok(())
    }
    
    if msg.mentions.len() < 1 {
        msg.channel_id.say(ctx, "Please mention the user you want to mute!").await?;

        return Ok(())
    }

    let user_id = msg.mentions[0].id;

    let guild = msg.guild(ctx).await.unwrap();
    let mut member = guild.member(ctx, user_id).await?;

    let mute_info = handle_mute_role(ctx, &guild, Some(msg.channel_id)).await?;

    if !member.roles.contains(&mute_info.mute_role_id) {
        msg.channel_id.say(ctx, format!("{} is not muted!", user_id.mention())).await?;

        return Ok(())
    }

    {
        let data = ctx.data.read().await;
        let mute_map = data.get::<MuteMap>().unwrap();
        if let Some(mute_guard) = mute_map.get(&(guild.id, user_id)) {
            mute_guard.value().abort();
        }
    }

    member.remove_role(ctx, mute_info.mute_role_id).await?;

    let mute_embed = embed_store::get_mute_embed(&msg.mentions[0], false, false, None);
    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.0 = mute_embed.0;
            e
        })
    }).await?;

    Ok(())
}

async fn prepare_mute_timer(ctx: &Context, user_id: UserId, guild_id: GuildId, mute_time_num: u64) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards?");

    let future_time = current_time.as_secs() + mute_time_num;

    sqlx::query!("INSERT INTO mutes
            VALUES($1, $2, $3)
            ON CONFLICT (guild_id, user_id)
            DO UPDATE
            SET mute_time = EXCLUDED.mute_time", 
            guild_id.0 as i64, user_id.0 as i64, future_time as i64)
        .execute(pool).await?;

    let ctx_clone = ctx.clone();
    
    tokio::spawn(async move {
        create_mute_timer(ctx_clone, mute_time_num, user_id, guild_id).await;
    });

    Ok(())
}

async fn create_mute_timer(ctx: Context, time: u64, user_id: UserId, guild_id: GuildId) {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let future = Abortable::new(unmute_by_time(&ctx, &user_id, &guild_id), abort_registration);

    let data = ctx.data.read().await;
    let mute_map = data.get::<MuteMap>().unwrap();
    mute_map.insert((guild_id, user_id), abort_handle);

    delay_for(Duration::from_secs(time)).await;
    match future.await {
        Ok(_) => {},
        Err(_e) => {
            let pool = data.get::<ConnectionPool>().unwrap();

            if let Err(e) = sqlx::query!("DELETE FROM mutes WHERE guild_id = $1 AND user_id = $2", guild_id.0 as i64, user_id.0 as i64)
                    .execute(pool).await {
                eprintln!("Error when deleting mute entry from user {} in guild {}: {}", user_id.0, guild_id.0, e);
            }
        }
    }
}

async fn unmute_by_time(ctx: &Context, user_id: &UserId, guild_id: &GuildId) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();
    let guild = ctx.cache.guild(guild_id).await.unwrap();
    let mut member = guild.member(ctx, user_id).await?;

    let mute_info = handle_mute_role(ctx, &guild, None).await?;

    if !member.roles.contains(&mute_info.mute_role_id) {
        return Ok(())
    }

    sqlx::query!("DELETE FROM mutes WHERE guild_id = $1 AND user_id = $2", guild.id.0 as i64, user_id.0 as i64)
        .execute(pool).await?;

    member.remove_role(ctx, mute_info.mute_role_id).await?;

    let mute_embed = embed_store::get_mute_embed(&user_id.to_user(ctx).await?, false, false, None);
    mute_info.mute_channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.0 = mute_embed.0;
            e
        })
    }).await?;

    Ok(())
}

#[command]
async fn mutechannel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        msg.channel_id.say(ctx, "You can't execute this command because you're not a moderator on this server!").await?;

        return Ok(())
    }

    let test_id = args.single::<String>().unwrap_or_default();

    let channel_id = match parse_channel(&test_id) {
        Some(channel_id) => ChannelId::from(channel_id),
        None => {
            msg.channel_id.say(ctx, "Please provide a mentioned channel!").await?;

            return Ok(())
        }
    };

    let guild_id = msg.guild_id.unwrap();

    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    sqlx::query!("UPDATE guild_info SET mute_channel_id = $1 WHERE guild_id = $2", channel_id.0 as i64, guild_id.0 as i64)
        .execute(pool).await?;

    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.color(0xa5f2f3);
            e.title("New Mute Channel");
            e.description(format!("New channel: {}", channel_id.mention()));
            e
        })
    }).await?;

    Ok(())
}

async fn handle_mute_role(ctx: &Context, guild: &Guild, channel_id: Option<ChannelId>) -> Result<MuteInfo, Box<dyn std::error::Error + Send + Sync>> {
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let mute_data = sqlx::query!("SELECT muted_role_id, mute_channel_id FROM guild_info WHERE guild_id = $1", guild.id.0 as i64)
        .fetch_one(pool).await?;
    
    let channel_id = match channel_id {
        Some(id) => id,
        None  => ChannelId::from(mute_data.mute_channel_id.unwrap() as u64)
    };

    if mute_data.muted_role_id.is_none() {
        let mut new_mute_string = String::new();
        new_mute_string.push_str("Created a new role called `Muted`. \n");
        new_mute_string.push_str("Feel free to customize this role as much as you want \n");
        new_mute_string.push_str("If you accidentally delete this role, a new one will be created \n");
        new_mute_string.push_str("All channels have been updated with the mute role \n");
        new_mute_string.push_str("Use `mutechannel` to change where timed unmutes are sent");

        channel_id.say(ctx, new_mute_string).await?;

        return Ok(new_mute_role(ctx, guild, channel_id).await?)
    }

    let mute_role_id = RoleId::from(mute_data.muted_role_id.unwrap() as u64);

    if guild.roles.contains_key(&mute_role_id) {
        let mute_info = MuteInfo {
            mute_role_id,
            mute_channel_id: channel_id
        };

        return Ok(mute_info)
    } else {
        channel_id.say(ctx, 
            "You deleted the mute role from your server, but the database wasn't updated! Recreating role...").await?;
        
        return Ok(new_mute_role(ctx, guild, channel_id).await?)
    }
}

async fn new_mute_role(ctx: &Context, guild: &Guild, channel_id: ChannelId) -> Result<MuteInfo, Box<dyn std::error::Error + Send + Sync>> {
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let mute_role = guild.create_role(ctx, |r| {
        r.name("muted");
        r.permissions(Permissions::READ_MESSAGES | Permissions::READ_MESSAGE_HISTORY);
        r
    }).await?;

    let overwrite_voice = PermissionOverwrite {
        allow: Permissions::empty(),
        deny: Permissions::CONNECT | Permissions::SPEAK | Permissions::STREAM,
        kind: PermissionOverwriteType::Role(mute_role.id)
    };

    let overwrite_text = PermissionOverwrite {
        allow: Permissions::empty(),
        deny: Permissions::SEND_MESSAGES | Permissions::SEND_TTS_MESSAGES,
        kind: PermissionOverwriteType::Role(mute_role.id)
    };

    for channel in guild.channels.to_owned() {
        if channel.1.kind == ChannelType::Voice {
            channel.1.create_permission(ctx, &overwrite_voice).await?;
        } else {
            channel.1.create_permission(ctx, &overwrite_text).await?;
        }
    }

    sqlx::query!("UPDATE guild_info SET muted_role_id = $1, mute_channel_id = $2 WHERE guild_id = $3", mute_role.id.0 as i64, channel_id.0 as i64, guild.id.0 as i64)
        .execute(pool).await?;

    let mute_info = MuteInfo {
        mute_role_id: mute_role.id,
        mute_channel_id: channel_id 
    };

    Ok(mute_info)
}

pub async fn load_mute_timers(ctx: Context) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let timer_data = sqlx::query!("SELECT guild_id, user_id, mute_time FROM mutes")
        .fetch_all(pool).await?;

    for i in timer_data {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards?");

        let mute_time_diff = i.mute_time - current_time.as_secs() as i64;

        let guild_id = GuildId::from(i.guild_id as u64);
        let user_id = UserId::from(i.user_id as u64);

        if mute_time_diff <= 0 {
            unmute_by_time(&ctx, &user_id, &guild_id).await?;
        } else {
            let ctx_clone = ctx.clone();
            
            tokio::spawn(async move {
                create_mute_timer(ctx_clone, mute_time_diff as u64, user_id, guild_id).await;
            });
        }
    }

    Ok(())
}