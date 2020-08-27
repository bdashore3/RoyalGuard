use serenity::{
    prelude::*,
    model::prelude::*,
    framework::standard::{
        CommandResult,
        macros::command, Args
    }
};
use crate::helpers::{embed_store, permissions_helper};
use std::borrow::Cow;

#[command]
async fn kick(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        msg.channel_id.say(ctx, "You can't execute this command because you're not a moderator on this server!").await?;

        return Ok(())
    }

    let guild_id = msg.guild_id.unwrap();

    if args.is_empty() {
        msg.channel_id.say(ctx, "Please provide a user/id to kick!").await?;

        return Ok(())
    }

    let use_id = args.parse::<u64>().is_ok();

    let kick_user_id = match args.single::<UserId>() {
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

    if kick_user_id == msg.author.id {
        msg.channel_id.say(ctx, "Kicking yourself out isn't a good idea.").await?;

        return Ok(())
    }

    if permissions_helper::check_moderator(ctx, msg, Some(kick_user_id)).await? {
        msg.channel_id.say(ctx, "I can't kick an administrator/moderator! Please demote the user then try again.").await?;

        return Ok(())
    }

    let user = if use_id {
        Cow::Owned(kick_user_id.to_user(ctx).await?)
    } else {
        Cow::Borrowed(&msg.mentions[0])
    };

    let reason = if args.is_empty() {
        format!("{}#{}: No reason given", msg.author.name, msg.author.discriminator)
    } else {
        format!("{}#{}: {}", msg.author.name, msg.author.discriminator, args.rest())
    };

    if reason.chars().count() > 500 {
        msg.channel_id.say(ctx, "The reason has to be less than 500 characters!").await?;

        return Ok(())
    }

    match guild_id.kick_with_reason(ctx, kick_user_id, &reason).await {
        Ok(_) => {
            let kick_embed = embed_store::get_kick_embed(&user, &reason, use_id);

            msg.channel_id.send_message(ctx, |m| {
                m.embed(|e| {
                    e.0 = kick_embed.0;
                    e
                })
            }).await?;
        },
        Err(e) => {
            msg.channel_id.say(ctx, "Kick unsuccessful. The user must be in the guild and the bot must be above the user's role!").await?;

            eprintln!("Kick Error in guild {}: {}", guild_id.0, e);
        }
    };

    Ok(())
}

pub async fn kick_help(ctx: &Context, channel_id: ChannelId) {
    let mut content = String::new();
    content.push_str("kick <reason>: Kicks a user with an optional reason. \n\n");
    
    let _ = channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Kick help");
            e.description("Description: Kicks a user from the server");
            e.field("Commands", content, false);
            e
        })
    }).await;
}
