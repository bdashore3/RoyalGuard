use std::time::Duration;

use crate::ConnectionPool;
use serenity::{framework::standard::CommandResult, model::prelude::*, prelude::*};
use tokio::time::sleep;

#[derive(Debug, Default)]
struct ReactionInfo {
    guild_id: GuildId,
    user_id: UserId,
    message_id: MessageId,
    channel_id: ChannelId,
    emoji: String,
}

pub async fn dispatch_event(ctx: &Context, rxn: &Reaction, remove: bool) -> CommandResult {
    let wrapped_emoji = match &rxn.emoji {
        ReactionType::Unicode(name) => Some(name.to_owned()),
        #[allow(unused_variables)]
        ReactionType::Custom { name, id, animated } => {
            let i64_id = id.as_u64();
            Some(i64_id.to_string())
        }
        _ => None,
    };

    let reaction_info = ReactionInfo {
        guild_id: rxn.guild_id.unwrap(),
        user_id: rxn.user_id.unwrap(),
        message_id: rxn.message_id,
        channel_id: rxn.channel_id,
        emoji: {
            match wrapped_emoji {
                Some(emoji) => emoji,
                None => return Ok(()),
            }
        },
    };

    handle_role(ctx, remove, reaction_info).await?;

    Ok(())
}

async fn handle_role(ctx: &Context, remove: bool, rxn_info: ReactionInfo) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let guild_id = rxn_info.guild_id;
    let msg_id = rxn_info.message_id;

    let rxn_data = sqlx::query!(
        "SELECT role_id FROM reaction_roles WHERE guild_id = $1 AND message_id = $2 AND emoji = $3",
        guild_id.0 as i64,
        msg_id.0 as i64,
        rxn_info.emoji
    )
    .fetch_all(&pool)
    .await?;

    if !rxn_data.is_empty() {
        for data in rxn_data {
            let role_id = RoleId::from(data.role_id as u64);

            if remove {
                if ctx
                    .http
                    .remove_member_role(guild_id.0, rxn_info.user_id.0, role_id.0)
                    .await
                    .is_err()
                {
                    let err_msg = rxn_info.channel_id.say(ctx,
                        concat!("Role removal unsuccessful. Please make sure the bot's role is above the one you want to assign! \n",
                        "This message will delete itself in 10 seconds. Please report this to the moderators/administrators.")).await?;

                    sleep(Duration::from_secs(10)).await;

                    err_msg.delete(ctx).await?;
                };
            } else if ctx
                .http
                .add_member_role(guild_id.0, rxn_info.user_id.0, role_id.0)
                .await
                .is_err()
            {
                let err_msg = rxn_info.channel_id.say(ctx,
                    concat!("Role assignment unsuccessful. Please make sure the bot's role is above the one you want to assign! \n",
                    "This message will delete itself in 10 seconds. Please report this to the moderators/administrators.")).await?;

                sleep(Duration::from_secs(10)).await;

                err_msg.delete(ctx).await?;
            };
        }
    }

    Ok(())
}
