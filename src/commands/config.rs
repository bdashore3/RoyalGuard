use serenity::{
    prelude::*,
    model::prelude::*,
    framework::standard::{
        CommandResult,
        macros::command,
        Args
    }
};
use crate::{
    ConnectionPool,
    PubCreds,
    helpers::permissions_helper, structures::cmd_data::PrefixMap
};

/// Sets the prefix for the server using the first message argument
/// Execute this command with no arguments to get the current prefix
#[command]
#[only_in("guilds")]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();
    let prefixes = data.get::<PrefixMap>().unwrap();
    let default_prefix = data.get::<PubCreds>().unwrap().get("default prefix").unwrap().to_string();
    let guild_id = msg.guild_id.unwrap();
    let guild_name = msg.guild(ctx).await.unwrap().name;

    if args.is_empty() {
        let cur_prefix = match prefixes.get(&guild_id) {
            Some(prefix_guard) => prefix_guard.value().to_owned(),
            None => default_prefix
        };

        msg.channel_id.say(ctx, format!("My prefix for `{}` is `{}`", guild_name, cur_prefix)).await?;
        return Ok(())
    }
    
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(())
    }

    let new_prefix = args.single::<String>().unwrap();

    if new_prefix == default_prefix {
        sqlx::query!("UPDATE guild_info SET prefix = null WHERE guild_id = $1", guild_id.0 as i64)
            .execute(pool).await?;
        
        prefixes.remove(&guild_id);
    } else {
        sqlx::query!("UPDATE guild_info SET prefix = $1 WHERE guild_id = $2", new_prefix, guild_id.0 as i64)
            .execute(pool).await?;

        prefixes.insert(guild_id, new_prefix.to_string());
    }

    msg.channel_id.say(ctx, format!("My new prefix is `{}` for `{}`!", new_prefix, guild_name)).await?;

    Ok(())
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