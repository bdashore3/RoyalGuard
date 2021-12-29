use crate::{ConnectionPool, RoyalError};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
#[sub_commands(set, disable)]
async fn log(ctx: &Context, msg: &Message) -> CommandResult {
    logging_help(ctx, msg.channel_id).await;

    Ok(())
}

#[command]
async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel_id = match args.single::<String>() {
        Ok(raw_id) => raw_id.parse::<ChannelId>().unwrap_or(msg.channel_id),
        Err(_) => {
            msg.channel_id
                .say(
                    ctx,
                    RoyalError::MissingError("mentioned channel after the command"),
                )
                .await?;

            return Ok(());
        }
    };

    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    sqlx::query!(
        "INSERT INTO logging
            VALUES($1, $2, null, null)
            ON CONFLICT (guild_id)
            DO UPDATE SET message_channel_id = EXCLUDED.message_channel_id",
        msg.guild_id.unwrap().0 as i64,
        channel_id.0 as i64
    )
    .execute(&pool)
    .await?;

    msg.channel_id
        .say(
            ctx,
            format!(
                "The message logging channel has been successfully set to {}",
                channel_id.mention()
            ),
        )
        .await?;

    Ok(())
}

#[command]
async fn disable(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let logging_data = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM logging WHERE guild_id = $1)",
        msg.guild_id.unwrap().0 as i64
    )
    .fetch_one(&pool)
    .await?;

    if logging_data.exists.unwrap() {
        sqlx::query!(
            "DELETE FROM logging WHERE guild_id = $1",
            msg.guild_id.unwrap().0 as i64
        )
        .execute(&pool)
        .await?;

        msg.channel_id
            .say(ctx, "Logging has been disabled.")
            .await?;
    } else {
        msg.channel_id
            .say(ctx, "You haven't set a channel to log! Please set one.")
            .await?;
    }

    Ok(())
}

pub async fn logging_help(ctx: &Context, channel_id: ChannelId) {
    let cmd_content = "log <subcommand>: Base command to handle member logging setup. Use any of the subcommands below. \n\n";

    let sub_content = concat!(
        "set <channel ID>: Enables logging for the server with the provided channel ID to send logs to \n\n",
        "disable: Disables logging for the server.");

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Server logging help");
                e.description("Description: Working with message logs in a server");
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
