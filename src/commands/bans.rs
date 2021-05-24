use crate::helpers::{embed_store, permissions_helper};
use crate::{PermissionType, RoyalError};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
use std::borrow::Cow;

#[command]
async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(());
    }

    if args.is_empty() {
        ban_help(ctx, msg.channel_id).await;

        return Ok(());
    }

    let use_id = args.parse::<u64>().is_ok();

    let ban_user_id = match args.single::<UserId>() {
        Ok(user_id) => user_id,
        Err(_) => {
            msg.channel_id
                .say(ctx, RoyalError::MissingError("user mention/id"))
                .await?;

            return Ok(());
        }
    };

    if ban_user_id.to_user(ctx).await.is_err() {
        if use_id {
            msg.channel_id
                .say(ctx, RoyalError::MissingError("user id"))
                .await?;
        } else {
            msg.channel_id
                .say(ctx, RoyalError::MissingError("user mention"))
                .await?;
        }

        return Ok(());
    }

    if ban_user_id == msg.author.id {
        msg.channel_id
            .say(ctx, RoyalError::SelfError("ban"))
            .await?;

        return Ok(());
    }

    if permissions_helper::check_moderator(ctx, msg, Some(ban_user_id)).await? {
        msg.channel_id
            .say(
                ctx,
                RoyalError::PermissionError(PermissionType::Mention(
                    "ban",
                    "administrator/moderator",
                )),
            )
            .await?;

        return Ok(());
    }

    let user = if use_id {
        Cow::Owned(ban_user_id.to_user(ctx).await?)
    } else {
        Cow::Borrowed(&msg.mentions[0])
    };

    let reason = if args.is_empty() {
        format!(
            "{}#{}: No reason given",
            msg.author.name, msg.author.discriminator
        )
    } else {
        format!(
            "{}#{}: {}",
            msg.author.name,
            msg.author.discriminator,
            args.rest()
        )
    };

    if reason.chars().count() > 500 {
        msg.channel_id
            .say(ctx, "The reason has to be less than 500 characters!")
            .await?;

        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap();

    match guild_id.ban_with_reason(ctx, ban_user_id, 1, &reason).await {
        Ok(_) => {
            let ban_embed = embed_store::get_ban_embed(&user, &reason, use_id);

            msg.channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.0 = ban_embed.0;
                        e
                    })
                })
                .await?;
        }
        Err(e) => {
            msg.channel_id
                .say(ctx, RoyalError::UnsuccessfulError("Ban"))
                .await?;

            eprintln!("Ban Error in guild {}: {}", guild_id.0, e);
        }
    }

    Ok(())
}

#[command]
async fn unban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(());
    }

    if args.is_empty() {
        ban_help(ctx, msg.channel_id).await;

        return Ok(());
    }

    let use_id = args.parse::<u64>().is_ok();

    let user_id = match args.single::<UserId>() {
        Ok(user_id) => user_id,
        Err(_) => {
            if use_id {
                msg.channel_id
                    .say(ctx, RoyalError::MissingError("user id"))
                    .await?;
            } else {
                msg.channel_id
                    .say(ctx, RoyalError::MissingError("user mention"))
                    .await?;
            }

            return Ok(());
        }
    };

    let user = user_id.to_user(ctx).await?;

    let guild_id = msg.guild_id.unwrap();

    match msg.guild_id.unwrap().unban(ctx, user_id).await {
        Ok(_) => {
            let unban_embed = embed_store::get_unban_embed(&user, use_id);

            msg.channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.0 = unban_embed.0;
                        e
                    })
                })
                .await?;
        }
        Err(e) => {
            msg.channel_id
                .say(ctx, "Unban unsuccessful. Is the user already unbanned?")
                .await?;

            eprintln!("Unban Error in guild {}: {}", guild_id.0, e);
        }
    }

    Ok(())
}

pub async fn ban_help(ctx: &Context, channel_id: ChannelId) {
    let content = concat!(
        "ban <mention or id> <reason>: Bans a user with a reason \n\n",
        "unban <mention or id>: Unbans the mentioned user or provided ID"
    );

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Ban help");
                e.description("Description: Commands for Banning/Unbanning in a server");
                e.field("Commands", content, false);
                e
            })
        })
        .await;
}
