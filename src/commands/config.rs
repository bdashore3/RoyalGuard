use std::sync::Arc;

use crate::{
    helpers::{database_helper, permissions_helper},
    structures::cmd_data::PrefixMap,
    ConnectionPool, PubCreds, RoyalError,
};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
#[sub_commands(restore)]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let (pool, prefixes, default_prefix) = {
        let data = ctx.data.read().await;

        let pool = data.get::<ConnectionPool>().cloned().unwrap();
        let prefixes = data.get::<PrefixMap>().unwrap().clone();
        let default_prefix = data
            .get::<PubCreds>()
            .unwrap()
            .get("default prefix")
            .cloned()
            .unwrap();

        (pool, prefixes, default_prefix)
    };
    let guild_id = msg.guild_id.unwrap();
    let guild_name = msg.guild(ctx).await.unwrap().name;

    if args.is_empty() {
        let cur_prefix = match prefixes.get(&guild_id) {
            Some(prefix_guard) => prefix_guard.value().to_owned(),
            None => default_prefix,
        };

        msg.channel_id
            .say(
                ctx,
                format!("My prefix for `{}` is `{}`", guild_name, cur_prefix),
            )
            .await?;
        return Ok(());
    }

    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(());
    }

    let new_prefix = args.single::<String>().unwrap();

    if new_prefix == default_prefix {
        sqlx::query!(
            "UPDATE guild_info SET prefix = null WHERE guild_id = $1",
            guild_id.0 as i64
        )
        .execute(&pool)
        .await?;

        prefixes.remove(&guild_id);
    } else {
        sqlx::query!(
            "UPDATE guild_info SET prefix = $1 WHERE guild_id = $2",
            new_prefix,
            guild_id.0 as i64
        )
        .execute(&pool)
        .await?;

        prefixes.insert(guild_id, new_prefix.to_owned());
    }

    msg.channel_id
        .say(
            ctx,
            format!("My new prefix is `{}` for `{}`!", new_prefix, guild_name),
        )
        .await?;

    Ok(())
}

#[command]
#[required_permissions("ADMINISTRATOR")]
async fn resetprefix(ctx: &Context, msg: &Message) -> CommandResult {
    let (pool, prefixes, default_prefix) = {
        let data = ctx.data.read().await;

        let pool = data.get::<ConnectionPool>().cloned().unwrap();
        let prefixes = data.get::<PrefixMap>().unwrap().clone();
        let default_prefix = data
            .get::<PubCreds>()
            .unwrap()
            .get("default prefix")
            .cloned()
            .unwrap();

        (pool, prefixes, default_prefix)
    };

    let guild_id = msg.guild_id.unwrap();

    if prefixes.contains_key(&guild_id) {
        prefixes.remove(&guild_id);

        sqlx::query!(
            "UPDATE guild_info SET prefix = null WHERE guild_id = $1",
            guild_id.0 as i64
        )
        .execute(&pool)
        .await?;
    }

    msg.channel_id
        .say(ctx, format!("Reset the prefix back to {}", default_prefix))
        .await?;

    Ok(())
}

#[command]
#[owners_only(true)]
async fn restore(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    {
        let mut data = ctx.data.write().await;
        let new_prefixes = database_helper::fetch_prefixes(&pool).await?;

        data.insert::<PrefixMap>(Arc::new(new_prefixes));
    }

    msg.channel_id
        .say(ctx, "Prefixes successfully restored!")
        .await?;

    Ok(())
}

#[command]
#[aliases("mod")]
#[sub_commands(remove)]
async fn moderator(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if !permissions_helper::check_administrator(ctx, msg, None).await? {
        return Ok(());
    }

    if args.is_empty() {
        config_help(ctx, msg.channel_id).await;

        return Ok(());
    }

    if msg.mention_roles.is_empty() {
        msg.channel_id
            .say(ctx, RoyalError::MissingError("role"))
            .await?;

        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap();
    let role_id = msg.mention_roles[0];

    let role = role_id.to_role_cached(ctx).await.unwrap();

    if role.has_permissions(
        Permissions::BAN_MEMBERS | Permissions::MANAGE_MESSAGES,
        false,
    ) {
        let pool = ctx
            .data
            .read()
            .await
            .get::<ConnectionPool>()
            .cloned()
            .unwrap();

        sqlx::query!(
            "UPDATE guild_info SET mod_role_id = $1 WHERE guild_id = $2",
            role_id.0 as i64,
            guild_id.0 as i64
        )
        .execute(&pool)
        .await?;

        msg.channel_id
            .say(ctx, "Moderator role sucessfully set!")
            .await?;
    } else {
        let part_1 = "Your specified role doesn't have the permissions `Ban Members` or `Manage Messages`! \n";
        let part_2 = "These are required for the bot to work!";
        msg.channel_id
            .say(ctx, format!("{}{}", part_1, part_2))
            .await?;
    }

    Ok(())
}

#[command]
#[aliases("clear")]
#[required_permissions("ADMINISTRATOR")]
async fn remove(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let mod_data = sqlx::query!(
        "SELECT mod_role_id FROM guild_info WHERE guild_id = $1",
        guild_id.0 as i64
    )
    .fetch_one(pool)
    .await?;

    if mod_data.mod_role_id.is_none() {
        msg.channel_id.say(ctx,
            "There's no moderator role configured! Please configure one before using this command.").await?;
    } else {
        sqlx::query!(
            "UPDATE guild_info SET mod_role_id = null WHERE guild_id = $1",
            guild_id.0 as i64
        )
        .execute(pool)
        .await?;

        msg.channel_id.say(ctx,
            "Your moderator role has sucessfully been cleared. Now, only administrators can execute mod-only commands.").await?;
    }

    Ok(())
}

pub async fn config_help(ctx: &Context, channel_id: ChannelId) {
    let content = concat!(
        "prefix <characters>: Sets the server's bot prefix \n\n",
        "moderator <role mention>: Sets the moderator role for the server. \n",
            "Defaults to anyone with the `administrator` permission \n*Alias: mod* \n\n",
        "moderator remove: Clears the moderator role for the server. Moderator subcommand \n*Alias: clear*");

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Configuration help");
                e.description("Description: Commands for configuring the bot");
                e.field("Commands", content, false);
                e
            })
        })
        .await;
}
