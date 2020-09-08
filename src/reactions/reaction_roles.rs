use serenity::{
    prelude::*,
    model::prelude::*,
    framework::standard::CommandResult
};
use crate::ConnectionPool;

#[derive(Debug, Default)]
struct ReactionInfo<'a> {
    guild_id: GuildId,
    user_id: UserId,
    message_id: MessageId,
    emoji: &'a str,
}

pub async fn dispatch_event(ctx: &Context, rxn: &Reaction, remove: bool) -> CommandResult {
    let mut reaction_info = ReactionInfo::default();

    reaction_info.guild_id = rxn.guild_id.unwrap();
    reaction_info.user_id = rxn.user_id.unwrap();
    reaction_info.message_id = rxn.message_id;

    match &rxn.emoji {
        ReactionType::Unicode(name) => {
            reaction_info.emoji = name;

            handle_role(ctx, remove, reaction_info).await?;    
        },
        #[allow(unused_variables)]
        ReactionType::Custom { name, id, animated} => {
            let i64_id = &id.as_u64().to_string();
            reaction_info.emoji = i64_id;

            handle_role(ctx, remove, reaction_info).await?;
        },
        _ => {}
    }

    Ok(())
}

async fn handle_role(ctx: &Context, remove: bool, rxn_info: ReactionInfo<'_>) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let guild_id = rxn_info.guild_id;
    let msg_id = rxn_info.message_id;

    let rxn_data = sqlx::query!(
            "SELECT role_id FROM reaction_roles WHERE guild_id = $1 AND message_id = $2 AND emoji = $3",
            guild_id.0 as i64, msg_id.0 as i64, rxn_info.emoji)
        .fetch_optional(pool).await?;

    if let Some(rxn_data) = rxn_data {
        let role_id = RoleId::from(rxn_data.role_id as u64);

        if remove {
            ctx.http.remove_member_role(guild_id.0, rxn_info.user_id.0, role_id.0).await?;
        } else {
            ctx.http.add_member_role(guild_id.0, rxn_info.user_id.0, role_id.0).await?;
        }
    }

    Ok(())
}
