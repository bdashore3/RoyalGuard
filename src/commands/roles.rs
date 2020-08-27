use serenity::{
    prelude::*,
    model::prelude::*,
    framework::standard::{
        CommandResult,
        macros::command
    }
};
use crate::{
    helpers::{
        permissions_helper,
        embed_store
    }, 
    structures::cmd_data::ConnectionPool
};

#[command]
#[sub_commands(set, get, remove, clear)]
async fn roles(_ctx: &Context, _msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
async fn set(ctx: &Context, msg: &Message) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        msg.channel_id.say(ctx, "You can't execute this command because you're not a moderator on this server!").await?;

        return Ok(())
    }

    if msg.mention_roles.is_empty() {
        msg.channel_id.say(ctx, "Please provide some role mentions for me to work with!").await?;

        return Ok(())
    }

    let guild_id = msg.guild_id.unwrap();

    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    for role_id in &msg.mention_roles {
        sqlx::query!("INSERT INTO welcome_roles
                VALUES($1, $2)
                ON CONFLICT (guild_id, role_id)
                DO UPDATE
                SET role_id = EXCLUDED.role_id",
                guild_id.0 as i64, role_id.0 as i64)
            .execute(pool).await?;
    }

    msg.channel_id.say(ctx, "New welcome roles sucessfully set!").await?;
    
    Ok(())
}

#[command]
async fn remove(ctx: &Context, msg: &Message) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        msg.channel_id.say(ctx, "You can't execute this command because you're not a moderator on this server!").await?;

        return Ok(())
    }

    if msg.mention_roles.is_empty() {
        msg.channel_id.say(ctx, "Please provide some role mentions for me to work with!").await?;

        return Ok(())
    }

    let guild_id = msg.guild_id.unwrap();

    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    for role_id in &msg.mention_roles {
        sqlx::query!("DELETE FROM welcome_roles WHERE guild_id = $1 AND role_id = $2",
                guild_id.0 as i64, role_id.0 as i64)
            .execute(pool).await?;
    }

    msg.channel_id.say(ctx, "Welcome roles sucessfully removed!").await?;

    Ok(())
}

#[command]
async fn clear(ctx: &Context, msg: &Message) -> CommandResult {
    if !permissions_helper::check_administrator(ctx, msg, None).await? {
        return Ok(())
    }

    let guild_id = msg.guild_id.unwrap();

    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    sqlx::query!("DELETE FROM welcome_roles WHERE guild_id = $1", guild_id.0 as i64)
        .execute(pool).await?;

    msg.channel_id.say(ctx, "Cleared all roles to be assigned on welcome. You will have to re-add them manually.").await?;

    Ok(())
}

#[command]
async fn get(ctx: &Context, msg: &Message) -> CommandResult {
    let mut role_ids: Vec<RoleId> = Vec::new();

    let guild_id = msg.guild_id.unwrap();

    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let role_data = sqlx::query!("SELECT role_id FROM welcome_roles WHERE guild_id = $1", guild_id.0 as i64)
        .fetch_all(pool).await?;
    
    if role_data.is_empty() {
        msg.channel_id.say(ctx, "There are currently no welcome roles in this server!").await?;

        return Ok(())
    }

    for i in role_data {
        role_ids.push(RoleId::from(i.role_id as u64));
    }

    let roles_embed = embed_store::get_welcome_roles_embed(role_ids);

    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.0 = roles_embed.0;
            e
        })
    }).await?;

    Ok(())
}

pub async fn welcome_roles_help(ctx: &Context, channel_id: ChannelId) {
    let mut content = String::new();
    content.push_str(
        "set <role mention>: Sets the roles to give the user on a welcome event. Make sure they're mentionable! Can add more than one mention. \n\n");
    content.push_str("remove <role mention>: Removes a role given on welcome. \n\n");
    content.push_str("clear: Removes all roles given on welcome. \n\n");
    content.push_str("get: Prints out all roles given on welcome.");
    
    let _ = channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Welcome roles subcategory");
            e.description("Description: Gives roles when a user joins the server (subcommand of welcome)");
            e.field("Sub-commands", content, false);
            e
        })
    }).await;
}
