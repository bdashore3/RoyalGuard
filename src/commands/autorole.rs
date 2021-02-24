use crate::{
    helpers::{embed_store, permissions_helper},
    ConnectionPool,
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
#[aliases("add")]
async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(());
    }

    let guild = msg.guild(ctx).await.unwrap();

    let role_ids = args.iter::<RoleId>().enumerate().map(|r| {
        match r.1 {
            Ok(role) if guild.roles.contains_key(&role) => Ok(role),
            Ok(role) => Err(format!("Please provide a valid role id for ID {} in position {}", role.0, r.0 + 1)),
            Err(_) => Err(format!("The argument in position {} couldn't be parsed! Check the role ID?", r.0 + 1)),
        }
    }).collect::<Result<Vec<RoleId>, String>>();

    let role_ids = match role_ids {
        Ok(roles) => roles,
        Err(err) => {
            msg.channel_id.say(ctx, err).await?;
            return Ok(());
        },
    };

    let guild_id = msg.guild_id.unwrap();

    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    for role_id in role_ids {
        sqlx::query!(
            "INSERT INTO welcome_roles
                VALUES($1, $2)
                ON CONFLICT (guild_id, role_id)
                DO UPDATE
                SET role_id = EXCLUDED.role_id",
            guild_id.0 as i64,
            role_id.0 as i64
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
async fn remove(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(());
    }

    let guild = msg.guild(ctx).await.unwrap();

    let role_ids = match concat_role_ids(&guild, args)? {
        Ok(roles) => roles,
        Err(err) => {
            msg.channel_id.say(ctx, err).await?;
            return Ok(());
        },
    };

    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    for role_id in role_ids {
        sqlx::query!(
            "DELETE FROM welcome_roles WHERE guild_id = $1 AND role_id = $2",
            guild.id.0 as i64,
            role_id.0 as i64
        )
        .execute(&pool)
        .await?;
    }

    msg.channel_id
        .say(ctx, "Given autoroles sucessfully removed!")
        .await?;

    Ok(())
}

fn concat_role_ids(guild: &Guild, mut args: Args) -> CommandResult<Result<Vec<RoleId>, String>> {
    let role_ids = args.iter::<RoleId>().enumerate().map(|r| {
        match r.1 {
            Ok(role) if guild.roles.contains_key(&role) => Ok(role),
            Ok(role) => Err(format!("Please provide a valid role id for ID {} in position {}", role.0, r.0)),
            Err(_) => Err(format!("The argument in position {} couldn't be parsed! Check the role ID?", r.0)),
        }
    }).collect::<Result<Vec<RoleId>, String>>();

    Ok(role_ids)
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

    let role_ids = role_data
        .iter()
        .map(|x| RoleId::from(x.role_id as u64))
        .collect::<Vec<RoleId>>();

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
