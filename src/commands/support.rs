use crate::{
    commands::{
        autorole::*, bans::*, config::*, kicks::*, logging::*, mutes::*, new_members::*, purges::*,
        reaction_roles::*, warns::*,
    },
    helpers::{botinfo::*, command_utils},
};
use serenity::{
    builder::CreateEmbed,
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
async fn help(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        if command_utils::check_mention_prefix(msg) {
            emergency_help_message(ctx, msg.channel_id).await;
        } else {
            default_help_message(ctx, msg.channel_id).await;
        }

        return Ok(());
    }

    let subcommand = args.single::<String>()?;

    match subcommand.as_str() {
        "ban" => ban_help(ctx, msg.channel_id).await,
        "warn" => warn_help(ctx, msg.channel_id).await,
        "mute" => mute_help(ctx, msg.channel_id).await,
        "welcome" => new_member_help(ctx, msg.channel_id).await,
        "leave" => new_member_help(ctx, msg.channel_id).await,
        "welcome_roles" => autorole_help(ctx, msg.channel_id).await,
        "config" => config_help(ctx, msg.channel_id).await,
        "kick" => kick_help(ctx, msg.channel_id).await,
        "purge" => purge_help(ctx, msg.channel_id).await,
        "reaction_roles" => reaction_role_help(ctx, msg.channel_id).await,
        "logging" => logging_help(ctx, msg.channel_id).await,
        _ => {}
    }

    Ok(())
}

async fn emergency_help_message(ctx: &Context, channel_id: ChannelId) {
    let content = concat!(
        "prefix <characters>: Sets the server's bot prefix \n\n",
        "resetprefix: Reset's the server's prefix back to the default one \n\n",
        "restoredb: Re-adds your guild into the bot's database. ",
        "Use this if you have a database error popping up on every command."
    );

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("RoyalGuard Emergency Help");
                e.description("You should only use this if you mess up your prefix!");
                e.field("Commands", content, false);
                e
            })
        })
        .await;
}

async fn default_help_message(ctx: &Context, channel_id: ChannelId) {
    let categories = concat!(
        "ban \n",
        "warn \n",
        "mute \n",
        "welcome \n",
        "leave \n",
        "autorole \n",
        "config \n",
        "purge \n",
        "reaction_roles \n",
        "logging \n"
    );

    let _ = channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("RoyalGuard Help");
                e.description(concat!(
                    "Help for the RoyalGuard Discord bot \n",
                    "Command parameters: <> is required and () is optional \n",
                    "Please use `help <subcategory>` to see that category's help \n\n",
                    "NOTE: Threads are now supported in RoyalGuard. \n"
                ));
                e.field("Subcategories", format!("```\n{}```", categories), false);
                e.footer(|f| {
                    f.text("Use the support command for any further help! \nUse the privacy command to see the privacy policy.");
                    f
                });
                e
            })
        })
        .await;
}

#[command]
async fn support(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("RoyalGuard Support");
                e.description("Need more help?");
                e.field("Support Server", "https://discord.gg/sYQxnuD7Fj", false);
                e.field(
                    "Github Issues",
                    "https://github.com/bdashore3/RoyalGuard/issues",
                    false,
                );
                e.field("Donate", "https://ko-fi.com/kingbri", false);
                e.footer(|f| {
                    f.text("Created with ❤️ by kingbri#6666");
                    f
                })
            })
        })
        .await?;

    Ok(())
}

#[command]
async fn privacy(ctx: &Context, msg: &Message) -> CommandResult {
    let mut eb = CreateEmbed::default();

    eb.title("Privacy Policy");
    eb.description("The privacy policy for RoyalGuard can be found in the below link: \nhttps://kingbri.dev/royalguard/privacy_policy");

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.0 = eb.0;
                e
            })
        })
        .await?;

    Ok(())
}

#[command]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    let mut eb = CreateEmbed::default();

    let guild_count = ctx.cache.guilds().len();
    let channel_count = ctx.cache.guild_channel_count();
    let user_count = ctx.cache.user_count();

    let guild_name = if guild_count < 2 { "guild" } else { "guilds" };

    let last_commit = get_last_commit(ctx).await?;
    let sys_info = get_system_info(ctx).await?;

    let mut story_string = String::new();
    story_string.push_str(&format!(
        "Currently running on commit [{}]({}) \n",
        &last_commit.sha[..7],
        last_commit.html_url
    ));
    story_string.push_str(&format!("Inside `{}` {} \n", guild_count, guild_name));
    story_string.push_str(&format!("With `{}` total channels \n", channel_count));
    story_string.push_str(&format!("Along with `{}` faithful users \n", user_count));
    story_string.push_str(&format!(
        "Consuming `{:.3} MB` of memory \n",
        sys_info.memory
    ));
    story_string.push_str(&format!("With a latency of `{}`", sys_info.shard_latency));

    eb.title("RoyalGuard is");
    eb.color(0xfda50f);
    eb.description(story_string);

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.0 = eb.0;
                e
            })
        })
        .await?;

    Ok(())
}
