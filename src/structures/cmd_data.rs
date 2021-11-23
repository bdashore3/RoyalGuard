use dashmap::DashMap;
use futures::future::AbortHandle;
use reqwest::Client as Reqwest;
use serenity::{
    client::bridge::gateway::ShardManager,
    model::id::{GuildId, UserId},
    prelude::{Mutex, TypeMapKey},
};
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};

// All command context data structures
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct PubCreds;

impl TypeMapKey for PubCreds {
    type Value = Arc<HashMap<String, String>>;
}

pub struct ConnectionPool;

impl TypeMapKey for ConnectionPool {
    type Value = PgPool;
}

pub struct MuteMap;

impl TypeMapKey for MuteMap {
    type Value = Arc<DashMap<(GuildId, UserId), AbortHandle>>;
}

pub struct PrefixMap;

impl TypeMapKey for PrefixMap {
    type Value = Arc<DashMap<GuildId, String>>;
}

pub struct BotId;

impl TypeMapKey for BotId {
    type Value = UserId;
}

pub struct ReqwestClient;

impl TypeMapKey for ReqwestClient {
    type Value = Arc<Reqwest>;
}

pub struct EmergencyCommands;

impl TypeMapKey for EmergencyCommands {
    type Value = Arc<Vec<String>>;
}
