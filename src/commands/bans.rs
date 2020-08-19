use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command, Args,
};
use crate::helpers::{
    embed_store,
    command_utils,
    permissions_helper
};
use std::borrow::Cow;

#[command]
#[required_permissions(BAN_MEMBERS)]
async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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

    if !permissions_helper::check_mentioned_permission(ctx, msg, user_id, Permissions::MANAGE_MESSAGES).await {
        msg.channel_id.say(ctx, "I can't ban an administrator/moderator! Please demote the user then try again.").await?;
        return Ok(())
    }
    
    let reason = if args.is_empty() {
        "No reason given"
    } else {
        args.rest()
    };

    let user = if use_id {
        Cow::Owned(command_utils::get_user(ctx, &user_id).await?)
    } else {
        Cow::Borrowed(&msg.mentions[0])
    };

    let ban_embed = embed_store::get_ban_embed(use_id, &user, reason);

    msg.guild_id.unwrap().ban(ctx, user_id, 0).await?;
    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.0 = ban_embed.0;
            e
        })
    }).await?;

    Ok(())
}

#[command]
#[required_permissions(BAN_MEMBERS)]
async fn unban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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

    let user = if use_id {
        Cow::Owned(command_utils::get_user(ctx, &user_id).await?)
    } else {
        Cow::Borrowed(&msg.mentions[0])
    };

    let unban_embed = embed_store::get_unban_embed(use_id, &user);

    msg.guild_id.unwrap().unban(ctx, user_id).await?;
    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.0 = unban_embed.0;
            e
        })
    }).await?;

    Ok(())
}