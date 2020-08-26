use serenity::{
    prelude::*,
    model::prelude::*,
    framework::standard::{
        CommandResult,
        macros::command
    }
};
use crate::{
    ConnectionPool, 
    helpers::{
        embed_store,
        permissions_helper
    }
};
use sqlx::PgPool;

#[command]
async fn warn(ctx: &Context, msg: &Message) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        msg.channel_id.say(ctx, "You can't execute this command because you're not a moderator on this server!").await?;

        return Ok(())
    }

    if msg.mentions.len() < 1 {
        msg.channel_id.say(ctx, "Please mention a user to warn!").await?;

        return Ok(())
    }

    let warn_user = &msg.mentions[0];

    if warn_user.id == msg.author.id {
        msg.channel_id.say(ctx, "I don't think you can warn yourself.").await?;

        return Ok(())
    }

    if permissions_helper::check_moderator(ctx, msg, Some(warn_user.id)).await? {
        msg.channel_id.say(ctx, "I can't warn an administrator/moderator! Please demote the user then try again.").await?;

        return Ok(())
    }

    let guild_id = msg.guild_id.unwrap();

    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let warn_number = match fetch_warn_number(pool, guild_id, msg.mentions[0].id).await? {
        Some(warn_number) => warn_number + 1,
        None => 1
    };

    if warn_number == 3 {
        if let Err(e) = guild_id.ban(ctx, msg.mentions[0].id, 0).await {
            msg.channel_id.say(ctx, "Ban unsuccessful. Make sure the bot's role is above the bannable ones!").await?;

            eprintln!("Ban Error in guild {}: {}", guild_id.0, e);
        };

        msg.channel_id.say(ctx, format!("That's 3 warns! {} is banned!", warn_user.name)).await?;

        let ban_embed = embed_store::get_ban_embed(false, warn_user, "Passed the warn limit");

        msg.channel_id.send_message(ctx, |m| {
            m.embed(|e| {
                e.0 = ban_embed.0;
                e
            })
        }).await?;
        
        sqlx::query!("DELETE FROM warns WHERE guild_id = $1 AND user_id = $2", msg.guild_id.unwrap().0 as i64, msg.mentions[0].id.0 as i64)
            .execute(pool).await?;
    } else {
        update_warn(pool, guild_id, warn_user.id, warn_number).await?;

        let warn_embed = embed_store::get_warn_embed(warn_user, warn_number, true);

        msg.channel_id.send_message(ctx, |m| {
            m.embed(|e| {
                e.0 = warn_embed.0;
                e
            })
        }).await?;
    }

    Ok(())
}

#[command]
async fn unwarn(ctx: &Context, msg: &Message) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        msg.channel_id.say(ctx, "You can't execute this command because you're not a moderator on this server!").await?;

        return Ok(())
    }

    if msg.mentions.len() < 1 {
        msg.channel_id.say(ctx, "Please mention a user to warn!").await?;

        return Ok(())
    }

    let warn_user = &msg.mentions[0];

    if warn_user.id == msg.author.id {
        msg.channel_id.say(ctx, "You can't unwarn yourself! Unless you want to be warned...").await?;

        return Ok(())
    }

    let guild_id = msg.guild_id.unwrap();

    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let warn_number = match fetch_warn_number(pool, guild_id, msg.mentions[0].id).await? {
        Some(warn_number) => warn_number - 1,
        None => {
            msg.channel_id.say(ctx, format!("`{}` has never been warned!", warn_user.name)).await?;

            return Ok(())
        }
    };

    if warn_number == 0 {
        sqlx::query!("DELETE FROM warns WHERE guild_id = $1 AND user_id = $2", guild_id.0 as i64, warn_user.id.0 as i64)
            .execute(pool).await?;
    } else {
        update_warn(pool, guild_id, warn_user.id, warn_number).await?;
    }

    let unwarn_embed = embed_store::get_warn_embed(warn_user, warn_number, false);

    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.0 = unwarn_embed.0;
            e
        })
    }).await?;

    Ok(())
}

#[command]
async fn warns(ctx: &Context, msg: &Message) -> CommandResult {
    let warn_user = if msg.mentions.len() < 1 {
                &msg.author
            } else {
                &msg.mentions[0]
            };

    let guild_id = msg.guild_id.unwrap();

    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let warn_number = match fetch_warn_number(pool, guild_id, warn_user.id).await? {
        Some(number) => number,
        None => 0
    };

    msg.channel_id.say(ctx, format!("{} currently has `{}` warns", warn_user.name, warn_number)).await?;

    Ok(())
}

async fn fetch_warn_number(pool: &PgPool, guild_id: GuildId, warn_user_id: UserId) -> Result<Option<i32>, Box<dyn std::error::Error + Send + Sync>> {
    let guild_id = guild_id.0 as i64;
    let warn_user_id = warn_user_id.0 as i64;

    let warn_data = sqlx::query!("SELECT warn_number FROM warns WHERE guild_id = $1 AND user_id = $2", guild_id, warn_user_id)
        .fetch_optional(pool).await?;

    let warn_number = match warn_data {
        Some(warn_data) => Some(warn_data.warn_number),
        None => None
    };

    Ok(warn_number)
}

async fn update_warn(pool: &PgPool, guild_id: GuildId, warn_user_id: UserId, warn_number: i32) -> CommandResult {
    let guild_id = guild_id.0 as i64;
    let warn_user_id = warn_user_id.0 as i64;

    sqlx::query!("INSERT INTO warns(guild_id, user_id, warn_number)
            VALUES($1, $2, $3)
            ON CONFLICT (guild_id, user_id)
            DO UPDATE
            SET warn_number = EXCLUDED.warn_number",
        guild_id, warn_user_id, warn_number)
        .execute(pool).await?;

    Ok(())
}

pub async fn warn_help(ctx: &Context, channel_id: ChannelId) {
    let mut content = String::new();
    content.push_str("warn <mention>: Adds a warn to the mentioned user \n\n");
    content.push_str("unwarn <mention>: Removes a warn from the mentioned user \n\n");
    content.push_str("warns <mention>, Gets the amount of warns for the mentioned user or yourself");
    
    let _ = channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Warn help");
            e.description("Description: Commands for warning in a server");
            e.field("Commands", content, false);
            e
        })
    }).await;
}