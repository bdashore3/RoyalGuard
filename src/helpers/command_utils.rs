use serenity::{client::Context, model::channel::Message};
use crate::structures::cmd_data::{PubCreds, PrefixMap};

pub async fn get_command_name<'a>(ctx: &Context, msg: &'a Message) -> &'a str {
    let data = ctx.data.read().await;
    let prefixes = data.get::<PrefixMap>().unwrap();
    let default_prefix = data.get::<PubCreds>().unwrap().get("default prefix").unwrap();
    let guild_id = msg.guild_id.unwrap();

    let prefix_length = match prefixes.get(&guild_id) {
        Some(prefix_guard) => prefix_guard.value().len(),
        None => default_prefix.len()
    };

    let words = msg.content.split_whitespace().collect::<Vec<&str>>();
    let command = words.get(0).unwrap();

    &command[prefix_length..]
}

pub fn get_time(initial_time: u64, parameter: &str) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let value = match parameter {
        "s" => initial_time,
        "m" => initial_time * 60,
        "h" => initial_time * 3600,
        "d" => initial_time * 86400,
        "w" => initial_time * 604800,
        _ => {
            return Err("Invalid parameter input".into())
        }
    };

    Ok(value)
}
