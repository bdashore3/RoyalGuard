use serenity::{
    prelude::*,
    model::prelude::*
};
use crate::ConnectionPool;
use std::borrow::Cow;

pub async fn check_administrator(ctx: &Context, msg: &Message, user_id: Option<UserId>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let channel = msg.channel(ctx).await.unwrap().guild().unwrap();
    let permissions = channel.permissions_for_user(ctx, user_id.unwrap_or(msg.author.id)).await?;

    if permissions.administrator() {
        Ok(true)
    } else {
        msg.channel_id.say(ctx, "You can't execute this command because you aren't an administrator!").await?;

        Ok(false)
    }
}

pub async fn check_moderator(ctx: &Context, msg: &Message, user_id: Option<UserId>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let channel = msg.channel(ctx).await.unwrap().guild().unwrap();
    let permissions = channel.permissions_for_user(ctx, user_id.unwrap_or(msg.author.id)).await?;

    if permissions.administrator() {
        return Ok(true)
    } else {
        let user = match user_id {
            Some(user_id) => {
                Cow::Owned(user_id.to_user(ctx).await?)
            },
            None => Cow::Borrowed(&msg.author)
        };

        return Ok(check_moderator_internal(ctx, msg, &user).await?);
    }
}

async fn check_moderator_internal(ctx: &Context, msg: &Message, user: &User) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let data = sqlx::query!("SELECT mod_role_id FROM guild_info WHERE guild_id = $1", msg.guild_id.unwrap().0 as i64)
        .fetch_one(pool).await?;

    if data.mod_role_id.is_none() {
        return Ok(false)
    }

    let role_id = RoleId::from(data.mod_role_id.unwrap() as u64);
    let role = match role_id.to_role_cached(ctx).await {
        Some(role) => role,
        None => {
            msg.channel_id.say(ctx, "The configured moderation role was deleted! Please reconfigure it!").await?;

            return Err("No role found".into())
        }
    };

    if !role.has_permissions(Permissions::BAN_MEMBERS | Permissions::MANAGE_MESSAGES, false) {

        msg.channel_id.say(ctx, 
            "The moderation role does not have the `Ban Members` or the `Manage Messages` permission! Please fix this!").await?;

        return Err("Invalid mod role perms".into())
    }

    Ok(user.has_role(ctx, msg.guild_id.unwrap(), RoleId::from(data.mod_role_id.unwrap() as u64)).await.unwrap())
}