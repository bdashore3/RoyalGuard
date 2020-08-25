use serenity::{
    client::bridge::gateway::ShardManager,
    prelude::{TypeMapKey, Mutex}, 
    model::id::{UserId, GuildId}
};
use std::{collections::HashMap, sync::Arc};
use sqlx::PgPool;
use dashmap::DashMap;
use futures::future::AbortHandle;

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
