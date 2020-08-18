use serenity::{
    client::bridge::gateway::ShardManager,
    prelude::{TypeMapKey, Mutex}
};
use std::{collections::HashMap, sync::Arc};
use sqlx::PgPool;

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