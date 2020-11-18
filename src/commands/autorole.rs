use crate::{
    helpers::{embed_store, permissions_helper},
    ConnectionPool, RoyalError,
};
use serenity::{
    framework::standard::Args,
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
#[sub_commands(set, get, remove, clear)]
async fn autorole(ctx: &Context, msg: &Message) -> CommandResult {
    autorole_help(ctx, msg.channel_id).await;

    Ok(())
}

#[command]
async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(());
    }

    let mut roles: Vec<u64> = Vec::new();

    for arg in args.iter::<u64>() {
        let role_id = match arg {
            Ok(id) => {
                let role_id = RoleId::from(id);

                if !msg.guild(ctx).await.unwrap().roles.contains_key(&role_id) {
                    msg.channel_id
                        .say(ctx, "Please provide a valid role id!")
                        .await?;

                    continue;
                }

                id
            }
            Err(_) => continue,
        };

        roles.push(role_id)
    }

    for i in &msg.mention_roles {
        if !roles.contains(&i.0) {
            roles.push(i.0);
        }
    }

    let guild_id = msg.guild_id.unwrap();

    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    for role_id in roles {
        sqlx::query!(
            "INSERT INTO welcome_roles
                VALUES($1, $2)
                ON CONFLICT (guild_id, role_id)
                DO UPDATE
                SET role_id = EXCLUDED.role_id",
            guild_id.0 as i64,
            role_id as i64
        )
        .execute(&pool)
        .await?;
    }

    msg.channel_id
        .say(ctx, "New autoroles sucessfully set!")
        .await?;

    Ok(())
}

#[command]
async fn remove(ctx: &Context, msg: &Message) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(());
    }

    if msg.mention_roles.is_empty() {
        msg.channel_id
            .say(ctx, RoyalError::MissingError("role mention(s)"))
            .await?;

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

    for role_id in &msg.mention_roles {
        sqlx::query!(
            "DELETE FROM welcome_roles WHERE guild_id = $1 AND role_id = $2",
            guild_id.0 as i64,
            role_id.0 as i64
        )
        .execute(&pool)
        .await?;
    }

    msg.channel_id
        .say(ctx, "Autoroles sucessfully removed!")
        .await?;

    Ok(())
}

#[command]
async fn clear(ctx: &Context, msg: &Message) -> CommandResult {
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

    sqlx::query!(
        "DELETE FROM welcome_roles WHERE guild_id = $1",
        guild_id.0 as i64
    )
    .execute(&pool)
    .await?;

    msg.channel_id
        .say(
            ctx,
            "Cleared all roles to be auto-assigned on welcome. You will have to re-add them manually.",
        )
        .await?;

    Ok(())
}

#[command]
#[aliases("list")]
async fn get(ctx: &Context, msg: &Message) -> CommandResult {
    let mut role_ids: Vec<RoleId> = Vec::new();

    let guild_id = msg.guild_id.unwrap();

    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let role_data = sqlx::query!(
        "SELECT role_id FROM welcome_roles WHERE guild_id = $1",
        guild_id.0 as i64
    )
    .fetch_all(&pool)
    .await?;

    if role_data.is_empty() {
        msg.channel_id
            .say(ctx, "There are currently no autoroles in this server!")
            .await?;

        return Ok(());
    }

    for i in role_data {
        role_ids.push(RoleId::from(i.role_id as u64));
    }

    let roles_embed = embed_store::get_welcome_roles_embed(role_ids);

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.0 = roles_embed.0;
                e
            })
        })
        .await?;

    Ok(())
}

pub async fn autorole_help(ctx: &Context, channel_id: ChannelId) {
    let content = concat!(
        "set <role mention>: Sets the roles to give the user on a welcome event. Make sure they're mentionable! Can add more than one mention. \n\n",
        "remove <role mention>: Removes a role given on welcome. \n\n",
        "clear: Removes all roles given on welcome. \n\n",
        "get: Prints out all roles given on welcome.
        Alias: list");

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Autoroles");
                e.description(
                    "Description: Automatically gives roles when a user joins the server",
                );
                e.field("Sub-commands", content, false);
                e
            })
        })
        .await;
}
