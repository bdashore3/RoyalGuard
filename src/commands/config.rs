use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
    Args
};
use sqlx::{self, PgPool};
use crate::{
    ConnectionPool,
    PubCreds,
    helpers::permissions_helper
};

/// Sets the prefix for the server using the first message argument
/// Execute this command with no arguments to get the current prefix
#[command]
#[only_in("guilds")]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();
    let default_prefix = data.get::<PubCreds>().unwrap().get("default prefix").unwrap().to_string();
    let guild_id = msg.guild_id.unwrap().0 as i64;
    let guild_name = msg.guild(ctx).await.unwrap().name;

    if args.is_empty() {
        let cur_prefix = get_prefix(pool, msg.guild_id.unwrap(), default_prefix).await?;

        msg.channel_id.say(ctx, format!("My prefix for `{}` is `{}`", guild_name, cur_prefix)).await?;
        return Ok(())
    }
    
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(())
    }

    let new_prefix = args.single::<String>().unwrap();

    if new_prefix == default_prefix {
        sqlx::query!("UPDATE guild_info SET prefix = null WHERE guild_id = $1", guild_id)
            .execute(pool).await?;
    }
    else {
        sqlx::query!("UPDATE guild_info SET prefix = $1 WHERE guild_id = $2", new_prefix, guild_id)
            .execute(pool).await?;
    }

    msg.channel_id.say(ctx, format!("My new prefix is `{}` for `{}`!", new_prefix, guild_name)).await?;

    Ok(())
}

pub async fn get_prefix(pool: &PgPool, guild_id: GuildId, default_prefix: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut cur_prefix = default_prefix;
    let guild_data = sqlx::query!("SELECT prefix FROM guild_info WHERE guild_id = $1", guild_id.0 as i64)
        .fetch_optional(pool).await?;
    
    if let Some(guild_data) = guild_data {
        if let Some(prefix) = guild_data.prefix {
            cur_prefix = prefix;
        }
    }

    Ok(cur_prefix)
}

#[command]
async fn moderator(ctx: &Context, msg: &Message) -> CommandResult {
    if !permissions_helper::check_administrator(ctx, msg, msg.author.id).await? {
        return Ok(())
    }

    let role_id = msg.mention_roles[0];

    let role = role_id.to_role_cached(ctx).await.unwrap();

    if role.has_permissions(Permissions::BAN_MEMBERS | Permissions::MANAGE_MESSAGES, false) {
        let data = ctx.data.read().await;
        let pool = data.get::<ConnectionPool>().unwrap();

        sqlx::query!("UPDATE guild_info SET mod_role_id = $1", role_id.0 as i64)
            .execute(pool).await?;

        msg.channel_id.say(ctx, "Moderator role sucessfully set!").await?;
    } else {
        let part_1 = "Your specified role doesn't have the permissions `Ban Members` or `Manage Messages`! \n";
        let part_2 = "These are required for the bot to work!";
        msg.channel_id.say(ctx, format!("{}{}", part_1, part_2)).await?;
    }

    Ok(())
}