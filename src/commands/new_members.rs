use crate::{
    helpers::{command_utils, embed_store, permissions_helper},
    ConnectionPool, RoyalError,
};
use serenity::{
    builder::CreateEmbed,
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::parse_channel,
};

#[command]
#[sub_commands(channel, set, get, clear, clean)]
async fn welcome(ctx: &Context, msg: &Message) -> CommandResult {
    new_member_help(ctx, msg.channel_id).await;

    Ok(())
}

#[command]
#[sub_commands(channel, set, get, clear, clean)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    new_member_help(ctx, msg.channel_id).await;

    Ok(())
}

#[command]
async fn channel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(());
    }

    let parameter = command_utils::get_command_name(ctx, msg).await;
    let test_id = args.single::<String>().unwrap_or_default();

    let channel_id = match parse_channel(test_id) {
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

    let channel_data = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM new_members WHERE guild_id = $1)",
        guild_id.0 as i64
    )
    .fetch_one(&pool)
    .await?;

    if channel_data.exists.unwrap() {
        sqlx::query!(
            "UPDATE new_members SET channel_id = $1 WHERE guild_id = $2",
            channel_id.0 as i64,
            guild_id.0 as i64
        )
        .execute(&pool)
        .await?;

        let new_channel_embed = embed_store::get_channel_embed(channel_id, "Welcome/Leave");

        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.0 = new_channel_embed.0;
                    e
                })
            })
            .await?;
    } else {
        msg.channel_id
            .say(
                ctx,
                format!(
                    "{} channel isn't set! Please set a welcome/leave message first!",
                    parameter
                ),
            )
            .await?;
    }

    Ok(())
}

#[command]
async fn set(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(());
    }

    if args.is_empty() {
        msg.channel_id
            .say(ctx, RoyalError::MissingError("message"))
            .await?;

        return Ok(());
    }

    let parameter = command_utils::get_command_name(ctx, msg).await;
    let guild_id = msg.guild_id.unwrap();
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    match parameter {
        "welcome" => {
            sqlx::query!(
                "INSERT INTO new_members
                    VALUES($1, $2, $3, null)
                    ON CONFLICT (guild_id)
                    DO UPDATE
                    SET welcome_message = EXCLUDED.welcome_message",
                guild_id.0 as i64,
                msg.channel_id.0 as i64,
                args.rest()
            )
            .execute(&pool)
            .await?;
        }
        "leave" => {
            sqlx::query!(
                "INSERT INTO new_members
                    VALUES($1, $2, null, $3)
                    ON CONFLICT (guild_id)
                    DO UPDATE
                    SET leave_message = EXCLUDED.leave_message",
                guild_id.0 as i64,
                msg.channel_id.0 as i64,
                args.rest()
            )
            .execute(&pool)
            .await?;
        }
        _ => {
            return Err(format!(
                "Invalid command parameter \"{}\" in new member set!",
                parameter
            )
            .into())
        }
    }

    msg.channel_id
        .say(ctx, format!("`{}` message sucessfully set!", parameter))
        .await?;

    Ok(())
}

#[command]
#[aliases("list")]
async fn get(ctx: &Context, msg: &Message) -> CommandResult {
    let parameter = command_utils::get_command_name(ctx, msg).await;

    let guild_id = msg.guild_id.unwrap();
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let mut exists = false;
    let mut new_member_embed = CreateEmbed::default();

    match parameter {
        "welcome" => {
            let welcome_data = sqlx::query!(
                "SELECT welcome_message FROM new_members WHERE guild_id = $1",
                guild_id.0 as i64
            )
            .fetch_optional(&pool)
            .await?;

            if let Some(welcome_data) = welcome_data {
                if let Some(welcome_message) = welcome_data.welcome_message {
                    new_member_embed = embed_store::get_new_member_embed(
                        welcome_message,
                        msg.channel_id,
                        parameter,
                    );

                    exists = true;
                }
            }
        }
        "leave" => {
            let leave_data = sqlx::query!(
                "SELECT leave_message FROM new_members WHERE guild_id = $1",
                guild_id.0 as i64
            )
            .fetch_optional(&pool)
            .await?;

            if let Some(leave_data) = leave_data {
                if let Some(leave_messsage) = leave_data.leave_message {
                    new_member_embed = embed_store::get_new_member_embed(
                        leave_messsage,
                        msg.channel_id,
                        parameter,
                    );

                    exists = true;
                }
            }
        }
        _ => {
            return Err(format!(
                "Invalid command parameter \"{}\" in new member get!",
                parameter
            )
            .into())
        }
    }

    if exists {
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.0 = new_member_embed.0;
                    e
                })
            })
            .await?;
    } else {
        msg.channel_id
            .say(
                ctx,
                format!(
                    "The `{}` message doesn't exist! Did you not set it?",
                    parameter
                ),
            )
            .await?;
    }

    Ok(())
}

#[command]
async fn clear(ctx: &Context, msg: &Message) -> CommandResult {
    if !permissions_helper::check_administrator(ctx, msg, None).await? {
        return Ok(());
    }

    let parameter = command_utils::get_command_name(ctx, msg).await;

    let guild_id = msg.guild_id.unwrap();
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let mut exists = false;

    match parameter {
        "welcome" => {
            let welcome_data = sqlx::query!(
                "SELECT welcome_message FROM new_members WHERE guild_id = $1",
                guild_id.0 as i64
            )
            .fetch_optional(&pool)
            .await?;

            if let Some(welcome_data) = welcome_data {
                if welcome_data.welcome_message.is_some() {
                    sqlx::query!(
                        "UPDATE new_members SET welcome_message = null WHERE guild_id = $1",
                        guild_id.0 as i64
                    )
                    .execute(&pool)
                    .await?;

                    exists = true;
                }
            }
        }
        "leave" => {
            let leave_data = sqlx::query!(
                "SELECT leave_message FROM new_members WHERE guild_id = $1",
                guild_id.0 as i64
            )
            .fetch_optional(&pool)
            .await?;

            if let Some(leave_data) = leave_data {
                if leave_data.leave_message.is_some() {
                    sqlx::query!(
                        "UPDATE new_members SET leave_message = null WHERE guild_id = $1",
                        guild_id.0 as i64
                    )
                    .execute(&pool)
                    .await?;

                    exists = true;
                }
            }
        }
        _ => {
            return Err(format!(
                "Invalid command parameter \"{}\" in new member clear!",
                parameter
            )
            .into())
        }
    }

    if exists {
        msg.channel_id
            .say(ctx, format!("`{}` message sucessfully cleared!", parameter))
            .await?;
    } else {
        msg.channel_id
            .say(
                ctx,
                format!(
                    "The `{}` message doesn't exist! Did you not set it?",
                    parameter
                ),
            )
            .await?;
    }

    Ok(())
}

#[command]
#[aliases("purge")]
async fn clean(ctx: &Context, msg: &Message) -> CommandResult {
    if !permissions_helper::check_administrator(ctx, msg, None).await? {
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap();

    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let new_member_data = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM new_members WHERE guild_id = $1)",
        guild_id.0 as i64
    )
    .fetch_one(&pool)
    .await?;

    if new_member_data.exists.unwrap() {
        sqlx::query!(
            "DELETE FROM new_members WHERE guild_id = $1",
            guild_id.0 as i64
        )
        .execute(&pool)
        .await?;

        msg.channel_id.say(ctx,
            "You have been wiped from the database. \nPlease run the welcome set or leave set command if you want to re-add the messages").await?;
    } else {
        msg.channel_id
            .say(
                ctx,
                "You haven't set up a welcome/leave message! Purge aborted.",
            )
            .await?;
    }

    Ok(())
}

pub async fn new_member_help(ctx: &Context, channel_id: ChannelId) {
    let cmd_content = concat!(
        "prefix <character>: Sets the server's bot prefix to a single character prefix \n\n",
        "moderator <role mention>: Sets the moderator role for the server. \nDefaults to anyone with the `administrator` permission");

    let sub_content = concat!(
        "channel <channel Id>: Sets the channel where the messages are sent. Default channel is where you inited. \n\n",
        "get: Gets the welcome/leave message
        Alias: list\n\n",
        "clear: Removes the current welcome OR leave message.", 
            "If you don't want to use RoyalGuard for welcome/leave messages, use purge or clearall! \n\n",
        "purge: Removes the welcome/leave database entry. ONLY use this if you don't want to use RoyalGuard for welcomes/leaves!");

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Welcome/Leave message help");
                e.description("Description: Setting server welcome/leave messages");
                e.field("Commands", cmd_content, false);
                e.field("Sub-commands", sub_content, false);
                e.footer(|f| {
                    f.text("Check welcome_roles help for roles assigned on welcome!");
                    f
                });
                e
            })
        })
        .await;
}
