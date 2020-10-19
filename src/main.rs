mod commands;
mod helpers;
mod structures;
mod reactions;

use std::{
    env,
    collections::{
        HashSet,
        HashMap
    },
    sync::{
        Arc,
        atomic::{Ordering, AtomicBool}
    }
};
use serenity::{async_trait, client::bridge::gateway::GatewayIntents, model::guild::GuildUnavailable, framework::standard::{
        StandardFramework,
        CommandError,
        DispatchError,
        macros::hook
    }, http::Http, model::{
        prelude::{
            Permissions,
            Message, User
        },
        gateway::Ready,
        guild::{
            Guild, Member
        },
        id::{ChannelId, GuildId, RoleId},
        channel::Reaction
    }, model::id::MessageId, prelude::*};
use structures::{
    cmd_data::*,
    commands::*,
    errors::*
};
use helpers::{database_helper, delete_buffer, command_utils};
use dashmap::DashMap;
use reqwest::Client as Reqwest;
use crate::{
    reactions::reaction_roles,
    commands::mutes::load_mute_timers
};

// Event handler for when the bot starts
struct Handler {
    run_loop: AtomicBool
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        if self.run_loop.load(Ordering::Relaxed) {
            self.run_loop.store(false, Ordering::Relaxed);

            println!("Loading mute timers!");
            if let Err(e) = load_mute_timers(&ctx).await {
                println!("Error when restoring mutes! {}", e);
            }

            println!("Starting guild deletion loop!");
            tokio::spawn(async move {
                if let Err(e) = delete_buffer::guild_removal_loop(ctx).await {
                    panic!("Delete buffer failed to start!: {}", e);
                };
            });
        }
    }

    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, mut new_member: Member) {
        if new_member.user.bot {
            return
        }

        let pool = ctx.data.read().await
            .get::<ConnectionPool>().cloned().unwrap();

        let welcome_data = match sqlx::query!("SELECT welcome_message, channel_id FROM new_members WHERE guild_id = $1", guild_id.0 as i64)
                .fetch_optional(&pool).await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error in fetching the welcome data: {}", e);
                return;
            }
        };
        
        if let Some(welcome_data) = welcome_data {
            if let Some(message) = welcome_data.welcome_message {
                let channel_id = ChannelId::from(welcome_data.channel_id as u64);

                let welcome_message = message
                    .replace("{user}", &new_member.user.mention())
                    .replace("{server}", &guild_id.name(&ctx).await.unwrap());

                let _ = channel_id.say(&ctx, welcome_message).await;
            }
        }

        let role_check = sqlx::query!("SELECT EXISTS(SELECT 1 FROM welcome_roles WHERE guild_id = $1)", guild_id.0 as i64)
            .fetch_one(&pool).await.unwrap();

        if role_check.exists.unwrap() {
            let welcome_roles = sqlx::query!("SELECT role_id FROM welcome_roles WHERE guild_id = $1", guild_id.0 as i64)
                .fetch_all(&pool).await.unwrap();

            for i in welcome_roles {
                if let Err(_) = new_member.add_role(&ctx, RoleId::from(i.role_id as u64)).await {
                    sqlx::query!("DELETE FROM welcome_roles WHERE guild_id = $1 AND role_id = $2", guild_id.0 as i64, i.role_id)
                        .execute(&pool).await.unwrap();
                }
            }
        }
    }

    async fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, _member_data_if_available: Option<Member>) {
        let pool = ctx.data.read().await
            .get::<ConnectionPool>().cloned().unwrap();

        if user.bot {
            return
        }

        let leave_data = match sqlx::query!("SELECT leave_message, channel_id FROM new_members WHERE guild_id = $1", guild_id.0 as i64)
            .fetch_optional(&pool).await {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error in fetching the welcome data: {}", e);
                    return;
                }
            };
        
        if let Some(leave_data) = leave_data {
            if let Some(message) = leave_data.leave_message {
                let channel_id = ChannelId::from(leave_data.channel_id as u64);

                let leave_message = message
                    .replace("{user}", &format!("**{}#{}**", user.name, user.discriminator))
                    .replace("{server}", &guild_id.name(&ctx).await.unwrap());

                let _ = channel_id.say(&ctx, leave_message).await;
            }
        }
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        let pool = ctx.data.read().await
            .get::<ConnectionPool>().cloned().unwrap();

        if let Err(e) = delete_buffer::add_new_guild(&pool, guild.id, is_new).await {
            eprintln!("Error in guild creation! (ID {}): {}", guild.id.0, e);
        }
    }

    async fn guild_delete(&self, ctx: Context, incomplete: GuildUnavailable, _full: Option<Guild>) {
        let pool = ctx.data.read().await
            .get::<ConnectionPool>().cloned().unwrap();

        if let Err(e) = delete_buffer::mark_for_deletion(&pool, incomplete.id).await {
            eprintln!("Error in marking for deletion! (ID {}): {}", incomplete.id.0, e);
        }
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        if let Err(e) = reaction_roles::dispatch_event(&ctx, &add_reaction, false).await {
            eprintln!("Error in reaction dispatch! (ID {}): {}", add_reaction.guild_id.unwrap().0, e);
        }
    }

    async fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        if let Err(e) = reaction_roles::dispatch_event(&ctx, &removed_reaction, true).await {
            eprintln!("Error in reaction dispatch! (ID {}): {}", removed_reaction.guild_id.unwrap().0, e);
        }
    }

    async fn reaction_remove_all(&self, ctx: Context, _channel_id: ChannelId, message_id: MessageId) {
        let pool = ctx.data.read().await
            .get::<ConnectionPool>().cloned().unwrap();

        if let Err(e) = delete_buffer::delete_leftover_reactions(&pool, message_id).await {
            println!("Error when deleting reactions in message delete! {}", e);
        }
    }

    async fn message_delete(&self, ctx: Context, _channel_id: ChannelId, message_id: MessageId) {
        let pool = ctx.data.read().await
            .get::<ConnectionPool>().cloned().unwrap();

        if let Err(e) = delete_buffer::delete_leftover_reactions(&pool, message_id).await {
            println!("Error when deleting reactions in message delete! {}", e);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    let creds = helpers::credentials_helper::read_creds(args[1].to_owned()).unwrap();
    let token = &creds.bot_token;

    let http = Http::new_with_token(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let pool = database_helper::obtain_db_pool(creds.db_connection).await?;

    let prefixes = database_helper::fetch_prefixes(&pool).await?;

    let reqwest_client = Reqwest::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:73.0) Gecko/20100101 Firefox/73.0")
        .build()?;

    let mut pub_creds = HashMap::new();
    pub_creds.insert("default prefix".to_owned(), creds.default_prefix);

    let emergency_commands = command_utils::get_allowed_commands();

    #[hook]
    async fn before(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
        if command_utils::check_mention_prefix(msg) {
            let emergency_commands = ctx.data.read().await
                .get::<EmergencyCommands>().cloned().unwrap();

            if emergency_commands.contains(&cmd_name.to_owned()) {
                let _ = msg.channel_id.say(ctx, 
                    format!("{}, you are running an emergency command!", msg.author.mention())).await;
                return true
            } else {
                return false
            }
        }

        true
    }

    // After a command is executed, go here
    #[hook]
    async fn after(ctx: &Context, msg: &Message, cmd_name: &str, error: Result<(), CommandError>) {
        if let Err(why) = error {
            let part_1 = "Looks like the bot encountered an error! \n";
            let part_2 = "Please use the `support` command and send the output to the support server!";
            let error_string = format!("{}{}", part_1, part_2);

            let _ = msg.channel_id.send_message(ctx, |m| {
                m.embed(|e| {
                    e.color(0xff69b4);
                    e.title("Aw Snap!");
                    e.description(error_string);
                    e.field("Command Name", cmd_name, false);
                    e.field("Error", format!("```{} \n```", why), false);
                    e
                })
            }).await;
        }
    }

    // On a dispatch error, go here. Catches most permission errors.
    #[hook]
    async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
        match error {
            DispatchError::LackingPermissions(Permissions::ADMINISTRATOR) => {
                let _ = msg.channel_id.say(ctx, 
                    RoyalError::PermissionError(PermissionType::SelfPerm("administrator"))).await;
            },
            DispatchError::NotEnoughArguments { min, given } => {
                let _ = msg.channel_id.say(ctx, format!("Args required: {}. Args given: {}", min, given)).await;
            },
            DispatchError::OnlyForOwners => {
                let _ = msg.channel_id.say(ctx, "This is a bot dev only command!").await;
            },
            _ => println!("Unhandled dispatch error: {:?}", error),
        }        
    }

    #[hook]
    async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
        let (prefixes, default_prefix) = {
            let data = ctx.data.read().await;
            let prefixes = data.get::<PrefixMap>().cloned().unwrap();
            let default_prefix = data.get::<PubCreds>().unwrap()
                .get("default prefix").cloned().unwrap();

            (prefixes, default_prefix)
        };

        let guild_id = msg.guild_id.unwrap();
 
        match prefixes.get(&guild_id) {
            Some(prefix_guard) => Some(prefix_guard.value().to_owned()),
            None => Some(default_prefix)
        }
    }

    // Link everything together!
    let framework = StandardFramework::new()
        .configure(|c| c
            .dynamic_prefix(dynamic_prefix)
            .on_mention(Some(bot_id))
            .owners(owners)
        )

        .on_dispatch_error(dispatch_error)
        .before(before)
        .after(after)

        .group(&GENERAL_GROUP)
        .group(&CONFIG_GROUP)
        .group(&GENERICMOD_GROUP)
        .group(&NEWMEMBERS_GROUP)
        .group(&SUPPORT_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler { run_loop: AtomicBool::new(true) } )
        .add_intent({
            let mut intents = GatewayIntents::all();
            intents.remove(GatewayIntents::DIRECT_MESSAGES);
            intents.remove(GatewayIntents::DIRECT_MESSAGE_REACTIONS);
            intents.remove(GatewayIntents::DIRECT_MESSAGE_TYPING);
            intents
        })
        .await
        .expect("Err creating client");

    {
        // Insert all structures into ctx data
        let mut data = client.data.write().await;

        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<PubCreds>(Arc::new(pub_creds));
        data.insert::<ConnectionPool>(pool);
        data.insert::<MuteMap>(Arc::new(DashMap::new()));
        data.insert::<PrefixMap>(Arc::new(prefixes));
        data.insert::<ReqwestClient>(Arc::new(reqwest_client));
        data.insert::<EmergencyCommands>(Arc::new(emergency_commands));
    }

    // Start up the bot! If there's an error, let the user know
    if let Err(why) = client.start_autosharded().await {
        eprintln!("Client error: {:?}", why);
    }

    Ok(())
}
