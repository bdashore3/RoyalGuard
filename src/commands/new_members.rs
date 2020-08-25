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
use crate::{helpers::{embed_store, command_utils, permissions_helper}, ConnectionPool};

#[command]
#[sub_commands(channel, set, get, clear, purge)]
async fn welcome(_ctx: &Context, _msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
#[sub_commands(channel, set, get, clear, purge)]
async fn leave(_ctx: &Context, _msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
async fn channel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        msg.channel_id.say(ctx, "You can't execute this command because you're not a moderator on this server!").await?;

        return Ok(())
    }

    let parameter = command_utils::get_command_name(ctx, msg).await;
    let test_id = args.single::<String>().unwrap_or_default();
            
    let channel_id = match parse_channel(test_id) {
        Some(channel_id) => ChannelId::from(channel_id),
        None => {
            msg.channel_id.say(ctx, "Please provide a mentioned channel!").await?;

            return Ok(())
        }
    };

    let guild_id = msg.guild_id.unwrap();
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let channel_data = sqlx::query!("SELECT EXISTS(SELECT 1 FROM new_members WHERE guild_id = $1)", guild_id.0 as i64)
        .fetch_one(pool).await?;

    if channel_data.exists.unwrap() {
        sqlx::query!("UPDATE new_members SET channel_id = $1 WHERE guild_id = $2", channel_id.0 as i64, guild_id.0 as i64)
            .execute(pool).await?;
        
        let new_channel_embed = embed_store::get_channel_embed(channel_id, "Welcome/Leave");

        msg.channel_id.send_message(ctx, |m| {
            m.embed(|e| {
                e.0 = new_channel_embed.0;
                e
            })
        }).await?;
    } else {
        msg.channel_id.say(ctx, format!("{} channel isn't set! Please set a welcome/leave message first!", parameter)).await?;
    }

    Ok(())
}

#[command]
async fn set(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        msg.channel_id.say(ctx, "You can't execute this command because you're not a moderator on this server!").await?;

        return Ok(())
    }

    if args.is_empty() {
        msg.channel_id.say(ctx, "Please provide a message that I can use!").await?;

        return Ok(())
    }

    let parameter = command_utils::get_command_name(ctx, msg).await;
    let guild_id = msg.guild_id.unwrap();
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    match parameter {
        "welcome" => {
            sqlx::query!("INSERT INTO new_members
                    VALUES($1, $2, $3, null)
                    ON CONFLICT (guild_id)
                    DO UPDATE
                    SET welcome_message = EXCLUDED.welcome_message",
                    guild_id.0 as i64, msg.channel_id.0 as i64, args.rest())
                .execute(pool).await?;
        },
        "leave" => {
            sqlx::query!("INSERT INTO new_members
                    VALUES($1, $2, null, $3)
                    ON CONFLICT (guild_id)
                    DO UPDATE
                    SET leave_message = EXCLUDED.leave_message",
                    guild_id.0 as i64, msg.channel_id.0 as i64, args.rest())
                .execute(pool).await?;
        },
        _ => {
            return Err(format!("Invalid command parameter \"{}\" in new member set!", parameter).into())
        }
    }

    msg.channel_id.say(ctx, format!("`{}` message sucessfully set!", parameter)).await?;

    Ok(())
}

#[command]
async fn get(ctx: &Context, msg: &Message) -> CommandResult {
    let parameter = command_utils::get_command_name(ctx, msg).await;

    let guild_id = msg.guild_id.unwrap();
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let mut exists = false;
    let mut new_member_embed = CreateEmbed::default();

    match parameter {
        "welcome" => {
            let welcome_data = sqlx::query!("SELECT welcome_message FROM new_members WHERE guild_id = $1", guild_id.0 as i64)
                .fetch_optional(pool).await?;

            if let Some(welcome_data) = welcome_data {
                if let Some(welcome_message) = welcome_data.welcome_message {
                    new_member_embed = embed_store::get_new_member_embed(parameter, welcome_message, msg.channel_id);

                    exists = true;
                }
            }
        },
        "leave" => {
            let leave_data = sqlx::query!("SELECT leave_message FROM new_members WHERE guild_id = $1", guild_id.0 as i64)
                .fetch_optional(pool).await?;

            if let Some(leave_data) = leave_data {
                if let Some(leave_messsage) = leave_data.leave_message {
                    new_member_embed = embed_store::get_new_member_embed(parameter, leave_messsage, msg.channel_id);

                    exists = true;
                }
            }
        },
        _ => {
            return Err(format!("Invalid command parameter \"{}\" in new member get!", parameter).into())
        }
    }

    if exists {
        msg.channel_id.send_message(ctx, |m| {
            m.embed(|e| {
                e.0 = new_member_embed.0;
                e
            })
        }).await?;
    } else {
        msg.channel_id.say(ctx, format!("The `{}` message doesn't exist! Did you not set it?", parameter)).await?;
    }

    Ok(())
}

#[command]
async fn clear(ctx: &Context, msg: &Message) -> CommandResult {
    if !permissions_helper::check_administrator(ctx, msg, None).await? {
        return Ok(())
    }

    let parameter = command_utils::get_command_name(ctx, msg).await;

    let guild_id = msg.guild_id.unwrap();
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let mut exists = false;

    match parameter {
        "welcome" => {
            let welcome_data = sqlx::query!("SELECT welcome_message FROM new_members WHERE guild_id = $1", guild_id.0 as i64)
                .fetch_optional(pool).await?;

            if let Some(welcome_data) = welcome_data {
                if welcome_data.welcome_message.is_some() {
                    sqlx::query!("UPDATE new_members SET welcome_message = null WHERE guild_id = $1", guild_id.0 as i64)
                        .execute(pool).await?;
                
                    exists = true;
                }
            }
        },
        "leave" => {
            let leave_data = sqlx::query!("SELECT leave_message FROM new_members WHERE guild_id = $1", guild_id.0 as i64)
                .fetch_optional(pool).await?;

            if let Some(leave_data) = leave_data {
                if leave_data.leave_message.is_some() {
                    sqlx::query!("UPDATE new_members SET leave_message = null WHERE guild_id = $1", guild_id.0 as i64)
                        .execute(pool).await?;
                    
                    exists = true;
                }
            }
        },
        _ => {
            return Err(format!("Invalid command parameter \"{}\" in new member clear!", parameter).into())
        }
    }

    if exists {
        msg.channel_id.say(ctx, format!("`{}` message sucessfully cleared!", parameter)).await?;
    } else {
        msg.channel_id.say(ctx, format!("The `{}` message doesn't exist! Did you not set it?", parameter)).await?;
    }

    Ok(())
}

#[command]
async fn purge(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let new_member_data = sqlx::query!("SELECT EXISTS(SELECT 1 FROM new_members WHERE guild_id = $1)", guild_id.0 as i64)
        .fetch_one(pool).await?;
    
    if new_member_data.exists.unwrap(){
        sqlx::query!("DELETE FROM new_members WHERE guild_id = $1", guild_id.0 as i64)
            .execute(pool).await?;

        msg.channel_id.say(ctx, 
            "You have been wiped from the database. \nPlease run the welcome set or leave set command if you want to re-add the messages").await?;
    } else {
        msg.channel_id.say(ctx, "You haven't set up a welcome/leave message! Purge aborted.").await?;
    }

    Ok(())
}