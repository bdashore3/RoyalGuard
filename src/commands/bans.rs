use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command, Args,
};
use crate::helpers::{
    embed_store,
    permissions_helper
};
use std::borrow::Cow;

#[command]
async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        msg.channel_id.say(ctx, "You can't execute this command because you're not a moderator on this server!").await?;

        return Ok(())
    }

    if args.len() < 1 {
        msg.channel_id.say(ctx, "Please provide a user/id to ban!").await?;

        return Ok(())
    }

    let use_id = 
        if args.parse::<u64>().is_ok() {
            true
        } else {
            false
        };

    let ban_user_id = match args.single::<UserId>() {
        Ok(user_id) => user_id,
        Err(_) => {
            if use_id {
                msg.channel_id.say(ctx, "Please provide a user id!").await?;
            } else {
                msg.channel_id.say(ctx, "Please provide a user mention!").await?;
            }

            return Ok(())
        }
    };

    if ban_user_id == msg.author.id {
        msg.channel_id.say(ctx, "I'm sorry, but you can't ban yourself.").await?;

        return Ok(())
    }

    if permissions_helper::check_moderator(ctx, msg, Some(ban_user_id)).await? {
        msg.channel_id.say(ctx, "I can't ban an administrator/moderator! Please demote the user then try again.").await?;

        return Ok(())
    }
    
    let reason = if args.is_empty() {
        "No reason given"
    } else {
        args.rest()
    };

    let user = if use_id {
        Cow::Owned(ban_user_id.to_user(ctx).await?)
    } else {
        Cow::Borrowed(&msg.mentions[0])
    };

    let guild_id = msg.guild_id.unwrap();

    match guild_id.ban(ctx, ban_user_id, 0).await {
        Ok(_) => {
            let ban_embed = embed_store::get_ban_embed(use_id, &user, reason);

            msg.channel_id.send_message(ctx, |m| {
                m.embed(|e| {
                    e.0 = ban_embed.0;
                    e
                })
            }).await?;
        }
        Err(e) => {
            msg.channel_id.say(ctx, "Ban unsuccessful. Make sure the bot's role is above the bannable ones!").await?;

            eprintln!("Ban Error in guild {}: {}", guild_id.0, e);
        }
    }

    Ok(())
}

#[command]
async fn unban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        msg.channel_id.say(ctx, "You can't execute this command because you're not a moderator on this server!").await?;
        return Ok(())
    }

    if args.len() < 1 {
        msg.channel_id.say(ctx, "Please provide a user/id to unban!").await?;
        return Ok(())
    }

    let use_id = 
        if args.parse::<u64>().is_ok() {
            true
        } else {
            false
        };

    let user_id = match args.single::<UserId>() {
        Ok(user_id) => user_id,
        Err(_) => {
            if use_id {
                msg.channel_id.say(ctx, "Please provide a user id!").await?;
            } else {
                msg.channel_id.say(ctx, "Please provide a user mention!").await?;
            }

            return Ok(())
        }
    };

    let user = user_id.to_user(ctx).await?;

    let guild_id = msg.guild_id.unwrap();

    match msg.guild_id.unwrap().unban(ctx, user_id).await {
        Ok(_) => {
            let unban_embed = embed_store::get_unban_embed(use_id, &user);

            msg.channel_id.send_message(ctx, |m| {
                m.embed(|e| {
                    e.0 = unban_embed.0;
                    e
                })
            }).await?;
        },
        Err(e) => {
            msg.channel_id.say(ctx, "Unban unsuccessful. Is the user already unbanned?").await?;

            eprintln!("Unban Error in guild {}: {}", guild_id.0, e);
        }
    }

    Ok(())
}