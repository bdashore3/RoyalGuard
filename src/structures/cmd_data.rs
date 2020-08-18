use serenity::{
    client::bridge::gateway::ShardManager,
    prelude::{TypeMapKey, Mutex}
};
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