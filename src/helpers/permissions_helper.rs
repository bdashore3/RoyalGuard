use serenity::{
    prelude::*,
    model::prelude::*
};
use crate::{ConnectionPool, RoyalError, PermissionType};
use std::borrow::Cow;

pub async fn check_administrator(ctx: &Context, msg: &Message, user_id: Option<UserId>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let channel = msg.channel(ctx).await.unwrap().guild().unwrap();
    let permissions = channel.permissions_for_user(ctx, user_id.unwrap_or(msg.author.id)).await?;

    if permissions.administrator() {
        Ok(true)
    } else {
        msg.channel_id.say(ctx, RoyalError::PermissionError(PermissionType::SelfPerm("administrator"))).await?;

        Ok(false)
    }
}

pub async fn check_moderator(ctx: &Context, msg: &Message, user_id: Option<UserId>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let channel = msg.channel(ctx).await.unwrap().guild().unwrap();
    let is_admin = channel.permissions_for_user(ctx, user_id.unwrap_or(msg.author.id)).await?.administrator();

    if is_admin {
        return Ok(true)
    } else {
        let user = match user_id {
            Some(user_id) => {
                Cow::Owned(user_id.to_user(ctx).await?)
            },
            None => Cow::Borrowed(&msg.author)
        };

        let mod_result = check_moderator_internal(ctx, msg, &user).await?;

        if user_id.is_none() && !mod_result {
            msg.channel_id.say(ctx, RoyalError::PermissionError(PermissionType::SelfPerm("moderator"))).await?;
        }

        return Ok(mod_result)
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
            let part_1 = "The configured moderation role was deleted from the server! Please reconfigure it! \n";
            let part_2 = "Defaulting to administrators \n";
            let part_3 = "If you don't want to see this message, an admin must use the command `moderator clear`";

            msg.channel_id.say(ctx, format!("{}{}{}", part_1, part_2, part_3)).await?;

            return Ok(false)
        }
    };

    if !role.has_permissions(Permissions::BAN_MEMBERS | Permissions::MANAGE_MESSAGES, false) {
            let part_1 = "The moderation role does not have the `Ban Members` or the `Manage Messages` permission! Please fix this! \n";
            let part_2 = "Defaulting to administrators \n";
            let part_3 = "If you don't want to see this message, an admin must use the command `moderator clear`";

            msg.channel_id.say(ctx, format!("{}{}{}", part_1, part_2, part_3)).await?;

        return Ok(false)
    }

    Ok(user.has_role(ctx, msg.guild_id.unwrap(), RoleId::from(data.mod_role_id.unwrap() as u64)).await.unwrap())
}
