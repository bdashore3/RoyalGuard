use serenity::{
    prelude::*,
    model::prelude::*,
    framework::standard::{
        CommandResult,
        macros::command, Args
    }, builder::CreateEmbed
};
use crate::{
    commands::{
        bans::*,
        config::*,
        mutes::*,
        new_members::*,
        roles::*,
        warns::*,
        kicks::*,
        purges::*,
        reaction_roles::*
    },
    helpers::{
        botinfo::*,
        command_utils
    }
};

#[command]
async fn help(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        if command_utils::check_mention_prefix(msg) {
            emergency_help_message(ctx, msg.channel_id).await;
        } else {
            default_help_message(ctx, msg.channel_id).await;
        }

        return Ok(())
    }

    let subcommand = args.single::<String>()?;

    match subcommand.as_str() {
        "ban" => ban_help(ctx, msg.channel_id).await,
        "warn" => warn_help(ctx, msg.channel_id).await,
        "mute" => mute_help(ctx, msg.channel_id).await,
        "welcome" => new_member_help(ctx, msg.channel_id).await,
        "leave" => new_member_help(ctx, msg.channel_id).await,
        "welcome_roles" => welcome_roles_help(ctx, msg.channel_id).await,
        "config" => config_help(ctx, msg.channel_id).await,
        "kick" => kick_help(ctx, msg.channel_id).await,
        "purge" => purge_help(ctx, msg.channel_id).await,
        "reaction_roles" => reaction_role_help(ctx, msg.channel_id).await,
        _ => {}
    }

    Ok(())
} 

async fn emergency_help_message(ctx: &Context, channel_id: ChannelId) {
    let content = concat!("prefix <characters>: Sets the server's bot prefix \n\n",
        "resetprefix: Reset's the server's prefix back to the default one");

    let _ = channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("RoyalGuard Emergency Help");
            e.description("You should only use this if you mess up your prefix!");
            e.field("Commands", content, false);
            e
        })
    }).await;
}

async fn default_help_message(ctx: &Context, channel_id: ChannelId) {
    let categories = concat!(
        "ban \n",
        "warn \n",
        "mute \n",
        "welcome \n",
        "leave \n",
        "welcome_roles \n",
        "config \n",
        "purge \n",
        "reaction_roles \n");

    let _ = channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("RoyalGuard Help");
            e.description(concat!("Help for the RoyalGuard Discord bot \n",
                "Command parameters: <> is required and () is optional \n",
                "Some new categories in v2.0 (config, reaction_roles)"));
            e.field("Subcategories", format!("```\n{}```", categories), false);
            e.footer(|f| {
                f.text("Use the support command for any further help!");
                f
            });
            e
        })
    }).await;
}

#[command]
async fn support(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("RoyalGuard Support");
            e.description("Need more help?");
            e.field("Support Server", "https://discord.gg/pswt7by", false);
            e.field("Github Issues", "https://github.com/bdashore3/RoyalGuard/issues", false);
            e.field("kingbri's twitter", "https://twitter.com/kingbri1st", false);
            e.footer(|f| {
                f.text("Created with ❤️ by kingbri#6666");
                f
            })
        })
    }).await?;

    Ok(())
}

#[command]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    let mut eb = CreateEmbed::default();

    let guild_count = ctx.cache.guilds().await.len();
    let channel_count = ctx.cache.guild_channel_count().await;
    let user_count = ctx.cache.user_count().await;

    let guild_name = if guild_count < 2 {
                "guild"
            } else {
                "guilds"
            };

    
    get_system_info(ctx).await;
    
    let last_commit = get_last_commit(ctx).await?;
    let sys_info = get_system_info(ctx).await;

    let mut story_string = String::new();
    story_string.push_str(&format!("Currently running on commit [{}]({}) \n", &last_commit.sha[..7], last_commit.html_url));
    story_string.push_str(&format!("Inside `{}` {} \n", guild_count, guild_name));
    story_string.push_str(&format!("With `{}` total channels \n", channel_count));
    story_string.push_str(&format!("Along with `{}` faithful users \n", user_count));
    story_string.push_str(&format!("Consuming `{:.3} MB` of memory \n", sys_info.memory));
    story_string.push_str(&format!("With a latency of `{}`", sys_info.shard_latency));

    eb.title("RoyalGuard is");
    eb.color(0xfda50f);
    eb.description(story_string);

    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.0 = eb.0;
            e
        })
    }).await?;

    Ok(())
}