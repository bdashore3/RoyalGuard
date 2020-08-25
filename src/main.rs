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
    sync::Arc,
};
use serenity::{
    async_trait,
    framework::standard::{
        StandardFramework,
        CommandError,
        DispatchError,
        macros::hook
    },
    http::Http,
    model::{
        prelude::{
            Permissions,
            Message, User
        },
        event::ResumedEvent,
        gateway::Ready,
        guild::{
            Guild, PartialGuild, Member
        },
        id::{ChannelId, GuildId}, 
    },
    prelude::*, 
    client::bridge::gateway::GatewayIntents
};
use structures::{
    cmd_data::*,
    commands::*
};
use helpers::database_helper;
use dashmap::DashMap;
use crate::commands::mutes::load_mute_timers;

// Event handler for when the bot starts
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        println!("Resumed");
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        println!("Loading mute timers!");
        if let Err(e) = load_mute_timers(ctx).await {
            println!("Error when restoring mutes! {}", e);
        }
    }

    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, new_member: Member) {
        let data = ctx.data.read().await;
        let pool = data.get::<ConnectionPool>().unwrap();

        let welcome_data = match sqlx::query!("SELECT welcome_message, channel_id FROM new_members WHERE guild_id = $1", guild_id.0 as i64)
                .fetch_optional(pool).await {
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
    }

    async fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, _member_data_if_available: Option<Member>) {
        let data = ctx.data.read().await;
        let pool = data.get::<ConnectionPool>().unwrap();

        let leave_data = match sqlx::query!("SELECT leave_message, channel_id FROM new_members WHERE guild_id = $1", guild_id.0 as i64)
            .fetch_optional(pool).await {
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

        let data = ctx.data.read().await;
        let pool = data.get::<ConnectionPool>().unwrap();
        let guild_id = guild.id.0 as i64;

        if is_new {
            sqlx::query!("INSERT INTO guild_info VALUES($1, null) ON CONFLICT DO NOTHING", guild_id)
                .execute(pool).await.unwrap();
        }
    }

    async fn guild_delete(&self, ctx: Context, incomplete: PartialGuild, _full: Option<Guild>) {
        
        let data = ctx.data.read().await;
        let pool = data.get::<ConnectionPool>().unwrap();
        let guild_id = incomplete.id.0 as i64;

        sqlx::query!("DELETE FROM guild_info WHERE guild_id = $1", guild_id)
            .execute(pool).await.unwrap();        
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    let creds = helpers::credentials_helper::read_creds(args[1].to_owned()).unwrap();
    let token = &creds.bot_token;

    let http = Http::new_with_token(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let pool = database_helper::obtain_db_pool(creds.db_connection).await?;

    let prefixes = database_helper::fetch_prefixes(&pool).await?;

    let mut pub_creds = HashMap::new();
    pub_creds.insert("default prefix".to_owned(), creds.default_prefix);

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
                    e.title("Oh Snap!");
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
                    "You can't execute this command because you aren't an administrator!").await;
            },
            DispatchError::NotEnoughArguments { min, given } => {
                let _ = msg.channel_id.say(ctx, format!("Args required: {}. Args given: {}", min, given)).await;
            },
            DispatchError::OnlyForOwners => {
                let _ = msg.channel_id.say(ctx, "This is a bot dev only command!").await;
            },
            DispatchError::IgnoredBot => {},
            _ => println!("Unhandled dispatch error: {:?}", error),
        }        
    }

    #[hook]
    async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
        let data = ctx.data.read().await;
        let prefixes = data.get::<PrefixMap>().unwrap();
        let guild_id = msg.guild_id.unwrap();

        match prefixes.get(&guild_id) {
            Some(prefix_guard) => Some(prefix_guard.value().to_owned()),
            None => None
        }
    }

    let prefix = pub_creds.get("default prefix").unwrap();

    // Link everything together!
    let framework = StandardFramework::new()
        .configure(|c| c
            .prefix(prefix)
            .dynamic_prefix(dynamic_prefix)
            .owners(owners)
        )

        .on_dispatch_error(dispatch_error)
        .after(after)

        .group(&GENERAL_GROUP)
        .group(&CONFIG_GROUP)
        .group(&GENERICMOD_GROUP)
        .group(&NEWMEMBERS_GROUP);

    let mut client = Client::new(&token)
        .framework(framework)
        .event_handler(Handler)
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
    }

    // Start up the bot! If there's an error, let the user know
    if let Err(why) = client.start_autosharded().await {
        eprintln!("Client error: {:?}", why);
    }

    Ok(())
}
