mod commands;
mod handlers;
mod helpers;
mod reactions;
mod structures;

use dashmap::DashMap;
use handlers::{event_handler::SerenityHandler, framework::get_framework};
use helpers::{command_utils, database_helper};
use reqwest::Client as Reqwest;
use serenity::{http::Http, model::prelude::UserId, prelude::GatewayIntents, Client};
use std::{
    collections::{HashMap, HashSet},
    env,
    sync::{atomic::AtomicBool, Arc},
};
use structures::{cmd_data::*, errors::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    let creds = helpers::credentials_helper::read_creds(args[1].to_owned()).unwrap();
    let token = creds.bot_token;

    let http = Http::new_with_application_id(&token, creds.application_id);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
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

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_BANS
        | GatewayIntents::GUILD_EMOJIS_AND_STICKERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .framework(get_framework(UserId(bot_id.0), owners).await)
        .event_handler(SerenityHandler {
            run_loop: AtomicBool::new(true),
        })
        .cache_settings(|settings| settings.max_messages(300))
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
        data.insert::<BotId>(UserId(bot_id.0));
        data.insert::<ReqwestClient>(Arc::new(reqwest_client));
        data.insert::<EmergencyCommands>(Arc::new(emergency_commands));
    }

    // Start up the bot! If there's an error, let the user know
    if let Err(why) = client.start_autosharded().await {
        eprintln!("Client error: {:?}", why);
    }

    Ok(())
}
