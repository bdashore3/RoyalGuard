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
            Message
        },
        event::ResumedEvent, 
        gateway::Ready, guild::{Guild, PartialGuild}, id::GuildId, 
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
        println!("Loading timers!");
        if let Err(e) = load_mute_timers(ctx).await {
            println!("Error when restoring mutes! {}", e);
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
    let creds = helpers::credentials_helper::read_creds(args[1].to_string()).unwrap();
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

    let mut pub_creds = HashMap::new();
    pub_creds.insert("default prefix".to_string(), creds.default_prefix);

    // After a command is executed, go here
    #[hook]
    async fn after(_: &Context, _: &Message, cmd_name: &str, error: Result<(), CommandError>) {
        if let Err(why) = error {
            println!("Error in {}: {:?}", cmd_name, why);
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
            _ => println!("Unhandled dispatch error: {:?}", error),
        }        
    }

    #[hook]
    async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
        let data = ctx.data.read().await;
        let pool = data.get::<ConnectionPool>().unwrap();
        let default_prefix = data.get::<PubCreds>().unwrap().get("default prefix").unwrap();
        let guild_id = msg.guild_id.unwrap();

        let cur_prefix = commands::config::get_prefix(pool, guild_id, default_prefix.to_string()).await.unwrap();

        Some(cur_prefix)
    }

    // Link everything together!
    let framework = StandardFramework::new()
        .configure(|c| c
            .dynamic_prefix(dynamic_prefix)
            .owners(owners)
        )

        .on_dispatch_error(dispatch_error)
        .after(after)

        .group(&GENERAL_GROUP)
        .group(&CONFIG_GROUP)
        .group(&GENERICMOD_GROUP);

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
    }

    // Start up the bot! If there's an error, let the user know
    if let Err(why) = client.start_autosharded().await {
        eprintln!("Client error: {:?}", why);
    }

    Ok(())
}
