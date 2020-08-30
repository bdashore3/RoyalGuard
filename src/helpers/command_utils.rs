use serenity::{
    client::Context, 
    model::{
        channel::Message,
        id::EmojiId
    }
};
use crate::structures::{
    EmojiIdentifier,
    cmd_data::{PubCreds, PrefixMap}
};
use regex::Regex;

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

pub fn check_mention_prefix(msg: &Message) -> bool {
    let words = msg.content.split_whitespace().collect::<Vec<&str>>();

    let re = Regex::new(r"<@!?\d+>").unwrap();

    re.is_match(words[0])
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

pub fn get_allowed_commands() -> Vec<String> {
    let allowed_commands: Vec<String> = vec![
        "prefix".to_owned(),
        "help".to_owned()
    ];

    allowed_commands
}

pub fn parse_emoji(mention: impl AsRef<str>) -> Option<EmojiIdentifier> {
    let mention = mention.as_ref();

    let len = mention.len();

    if len < 6 || len > 56 {
        return None;
    }

    if (mention.starts_with("<:") || mention.starts_with("<a:")) && mention.ends_with('>') {
        let mut name = String::default();
        let mut id = String::default();
        let animated = &mention[1..3] == "a:";

        let start = if animated { 3 } else { 2 };

        for (i, x) in mention[start..].chars().enumerate() {
            if x == ':' {
                let from = i + start + 1;

                for y in mention[from..].chars() {
                    if y == '>' {
                        break;
                    } else {
                        id.push(y);
                    }
                }

                break;
            } else {
                name.push(x);
            }
        }

        match id.parse::<u64>() {
            Ok(x) => Some(EmojiIdentifier {
                animated,
                name,
                id: EmojiId(x),
            }),
            _ => None,
        }
    } else {
        None
    }
}
