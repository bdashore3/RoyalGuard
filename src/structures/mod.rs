pub mod cmd_data;
pub mod commands;
pub mod errors;

use serde::Deserialize;
use serenity::model::id::EmojiId;
#[derive(Debug, Deserialize)]
pub struct CommitResponse {
    pub sha: String,
    pub html_url: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct EmojiIdentifier {
    pub animated: bool,
    pub id: EmojiId,
    pub name: String,
}

#[derive(Default, Debug)]
pub struct SysInfo {
    pub shard_latency: String,
    pub memory: f32,
}
