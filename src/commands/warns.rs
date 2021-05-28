use crate::{
    helpers::{embed_store, permissions_helper, warn_helper::*},
    ConnectionPool, PermissionType, RoyalError,
};
use serenity::{
    framework::standard::Args,
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
#[sub_commands(clear)]
async fn warn(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(());
    }

    if args.is_empty() {
        warn_help(ctx, msg.channel_id).await;

        return Ok(());
    }

    let warn_user_id = match args.single::<UserId>() {
        Ok(user_id) => user_id,
        Err(_) => {
            msg.channel_id
                .say(ctx, RoyalError::MissingError("user ID/mention"))
                .await?;

            return Ok(());
        }
    };

    let warn_user = match warn_user_id.to_user(ctx).await {
        Ok(user) => user,
        Err(_) => {
            msg.channel_id
                .say(ctx, RoyalError::MissingError("valid user ID/mention"))
                .await?;

            return Ok(());
        }
    };

    if warn_user.id == msg.author.id {
        msg.channel_id
            .say(ctx, RoyalError::SelfError("warn"))
            .await?;

        return Ok(());
    }

    if permissions_helper::check_moderator(ctx, msg, Some(warn_user.id)).await? {
        msg.channel_id
            .say(
                ctx,
                RoyalError::PermissionError(PermissionType::Mention(
                    "warn",
                    "administrator/moderator",
                )),
            )
            .await?;

        return Ok(());
    }

    let guild = msg.guild(ctx).await.unwrap();

    if !guild.members.contains_key(&warn_user.id) {
        msg.channel_id
            .say(ctx, RoyalError::MissingError("user ID/mention"))
            .await?;

        return Ok(());
    }

    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let warn_number = match fetch_warn_number(&pool, guild.id, warn_user.id).await? {
        Some(warn_number) => warn_number + 1,
        None => 1,
    };

    if warn_number == 3 {
        if let Err(e) = guild.id.ban(ctx, msg.mentions[0].id, 0).await {
            msg.channel_id
                .say(ctx, RoyalError::UnsuccessfulError("Ban"))
                .await?;

            eprintln!("Ban Error in guild {}: {}", guild.id.0, e);
        };

        msg.channel_id
            .say(
                ctx,
                format!("That's 3 warns! {} is banned!", warn_user.name),
            )
            .await?;

        let ban_embed =
            embed_store::get_ban_embed(&warn_user, "Self: Passed the warn limit", false);

        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.0 = ban_embed.0;
                    e
                })
            })
            .await?;

        sqlx::query!(
            "DELETE FROM warns WHERE guild_id = $1 AND user_id = $2",
            msg.guild_id.unwrap().0 as i64,
            msg.mentions[0].id.0 as i64
        )
        .execute(&pool)
        .await?;
    } else {
        update_warn(&pool, guild.id, warn_user.id, warn_number).await?;

        let warn_embed = embed_store::get_warn_embed(&warn_user, warn_number, true);

        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.0 = warn_embed.0;
                    e
                })
            })
            .await?;
    }

    Ok(())
}

#[command]
async fn unwarn(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(());
    }

    if args.is_empty() {
        warn_help(ctx, msg.channel_id).await;

        return Ok(());
    }

    let warn_user_id = match args.single::<UserId>() {
        Ok(user_id) => user_id,
        Err(_) => {
            msg.channel_id
                .say(ctx, RoyalError::MissingError("user ID/mention"))
                .await?;

            return Ok(());
        }
    };

    let warn_user = match warn_user_id.to_user(ctx).await {
        Ok(user) => user,
        Err(_) => {
            msg.channel_id
                .say(ctx, RoyalError::MissingError("valid user ID/mention"))
                .await?;

            return Ok(());
        }
    };

    let guild = msg.guild(ctx).await.unwrap();

    if !guild.members.contains_key(&warn_user.id) {
        msg.channel_id
            .say(ctx, RoyalError::MissingError("user ID/mention"))
            .await?;

        return Ok(());
    }

    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let warn_number = match fetch_warn_number(&pool, guild.id, warn_user.id).await? {
        Some(warn_number) => warn_number - 1,
        None => {
            msg.channel_id
                .say(ctx, format!("`{}` has never been warned!", warn_user.name))
                .await?;

            return Ok(());
        }
    };

    if warn_number == 0 {
        sqlx::query!(
            "DELETE FROM warns WHERE guild_id = $1 AND user_id = $2",
            guild.id.0 as i64,
            warn_user.id.0 as i64
        )
        .execute(&pool)
        .await?;
    } else {
        update_warn(&pool, guild.id, warn_user.id, warn_number).await?;
    }

    let unwarn_embed = embed_store::get_warn_embed(&warn_user, warn_number, false);

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.0 = unwarn_embed.0;
                e
            })
        })
        .await?;

    Ok(())
}

#[command]
async fn clear(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(());
    }

    if args.is_empty() {
        warn_help(ctx, msg.channel_id).await;

        return Ok(());
    }

    let warn_user_id = match args.single::<UserId>() {
        Ok(user_id) => user_id,
        Err(_) => {
            msg.channel_id
                .say(ctx, RoyalError::MissingError("user ID/mention"))
                .await?;

            return Ok(());
        }
    };

    let warn_user = match warn_user_id.to_user(ctx).await {
        Ok(user) => user,
        Err(_) => {
            msg.channel_id
                .say(ctx, RoyalError::MissingError("valid user ID/mention"))
                .await?;

            return Ok(());
        }
    };

    if warn_user.id == msg.author.id {
        msg.channel_id
            .say(ctx, RoyalError::SelfError("clear warns on"))
            .await?;

        return Ok(());
    }

    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let guild_id = msg.guild_id.unwrap();

    sqlx::query!(
        "DELETE FROM warns WHERE guild_id = $1 AND user_id = $2",
        guild_id.0 as i64,
        warn_user.id.0 as i64
    )
    .execute(&pool)
    .await?;

    let clear_embed = embed_store::get_warn_embed(&warn_user, 0, false);

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.0 = clear_embed.0;
                e
            })
        })
        .await?;

    Ok(())
}

#[command]
async fn warns(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild(ctx).await.unwrap();

    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    if let Ok(user_id) = args.single::<UserId>() {
        if !(guild.members.contains_key(&user_id)) {
            msg.channel_id
                .say(ctx, "This member doesn't exist in this server!")
                .await?;

            return Ok(());
        }

        let warn_number = fetch_warn_number(&pool, guild.id, user_id)
            .await?
            .unwrap_or(0);

        msg.channel_id
            .say(
                ctx,
                format!(
                    "{} currently has `{}` warn(s)",
                    user_id.mention(),
                    warn_number
                ),
            )
            .await?;
    } else if let Some(warns_string) = fetch_guild_warns(&pool, guild.id).await? {
        let warns_embed = embed_store::get_guild_warns_embed(guild.name, warns_string);

        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.0 = warns_embed.0;
                    e
                })
            })
            .await?;
    } else {
        msg.channel_id
            .say(ctx, "There are no warns in this server!")
            .await?;
    }

    Ok(())
}

pub async fn warn_help(ctx: &Context, channel_id: ChannelId) {
    let content = concat!(
        "warn <mention>: Adds a warn to the mentioned user \n\n",
        "unwarn <mention>: Removes a warn from the mentioned user \n\n",
        "warns <mention>, Gets the amount of warns for the mentioned user or yourself"
    );

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Warn help");
                e.description("Description: Commands for warning in a server");
                e.field("Commands", content, false);
                e
            })
        })
        .await;
}
