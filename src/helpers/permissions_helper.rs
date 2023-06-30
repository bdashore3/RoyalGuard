use crate::{ConnectionPool, PermissionType, RoyalError};
use serenity::{model::prelude::*, prelude::*};
use std::borrow::Cow;

pub async fn check_administrator(
    ctx: &Context,
    msg: &Message,
    user_id: Option<UserId>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let channel = msg
        .channel_id
        .to_channel(ctx)
        .await
        .unwrap()
        .guild()
        .unwrap();
    let permissions = match channel.permissions_for_user(ctx, user_id.unwrap_or(msg.author.id)) {
        Ok(permissions) => permissions,
        Err(_) => return Ok(false),
    };

    if permissions.administrator() {
        Ok(true)
    } else {
        msg.channel_id
            .say(
                ctx,
                RoyalError::PermissionError(PermissionType::SelfPerm("administrator")),
            )
            .await?;

        Ok(false)
    }
}

pub async fn check_moderator(
    ctx: &Context,
    msg: &Message,
    user_id: Option<UserId>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let channel = msg
        .channel_id
        .to_channel(ctx)
        .await
        .unwrap()
        .guild()
        .unwrap();
    let user_permissions = match channel.permissions_for_user(ctx, user_id.unwrap_or(msg.author.id))
    {
        Ok(permissions) => permissions,
        Err(_) => return Ok(false),
    };

    let is_admin = user_permissions.administrator();

    if is_admin {
        Ok(true)
    } else {
        let user = match user_id {
            Some(user_id) => Cow::Owned(user_id.to_user(ctx).await?),
            None => Cow::Borrowed(&msg.author),
        };

        let mod_result = check_moderator_internal(ctx, msg, &user).await?;

        if user_id.is_none() && !mod_result {
            msg.channel_id
                .say(
                    ctx,
                    RoyalError::PermissionError(PermissionType::SelfPerm("moderator")),
                )
                .await?;
        }

        Ok(mod_result)
    }
}

async fn check_moderator_internal(
    ctx: &Context,
    msg: &Message,
    user: &User,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let data = sqlx::query!(
        "SELECT mod_role_id FROM guild_info WHERE guild_id = $1",
        msg.guild_id.unwrap().0 as i64
    )
    .fetch_one(&pool)
    .await?;

    if data.mod_role_id.is_none() {
        return Ok(false);
    }

    let role_id = RoleId::from(data.mod_role_id.unwrap() as u64);
    let role = match role_id.to_role_cached(ctx) {
        Some(role) => role,
        None => {
            let response = concat!(
                "The configured moderation role was deleted from the server! Please reconfigure it! \n",
                "Defaulting to administrators \n",
                "If you don't want to see this message, an admin must use the command `moderator clear`"
            );

            msg.channel_id.say(ctx, response).await?;

            return Ok(false);
        }
    };

    if !role.has_permissions(
        Permissions::BAN_MEMBERS | Permissions::MANAGE_MESSAGES,
        false,
    ) {
        let response = concat!(
                "The moderation role does not have the `Ban Members` or the `Manage Messages` permission! Please fix this! \n",
                "Defaulting to administrators \n",
                "If you don't want to see this message, an admin must use the command `moderator clear`"
            );

        msg.channel_id.say(ctx, response).await?;

        return Ok(false);
    }

    let role_id = RoleId::from(data.mod_role_id.unwrap() as u64);
    let has_role = user
        .has_role(ctx, msg.guild_id.unwrap(), role_id)
        .await
        .unwrap_or(false);

    Ok(has_role)
}
