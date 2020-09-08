use serenity::{
    prelude::*,
    model::prelude::*,
    framework::standard::{
        CommandResult,
        macros::command,
        Args, Delimiter
    },
    utils::parse_channel
};
use crate::{
    RoyalError,
    ConnectionPool, 
    helpers::permissions_helper,
    helpers::command_utils::*
};
use unic_emoji_char::is_emoji;
use std::time::Duration;

#[derive(Debug, Default, Clone)]
struct WizardIntermediate {
    message_id: u64,
    channel_id: ChannelId,
    emoji: ReactionEmoji,
    role_id: RoleId
}

#[derive(Debug, Default, Clone)]
struct ReactionEmoji {
    emoji: Option<String>,
    animated: Option<bool>,
    name: Option<String>
}

#[command]
#[aliases("rr")]
#[sub_commands(new, remove, list, wizard)]
async fn reactionrole(ctx: &Context, msg: &Message) -> CommandResult {
    reaction_role_help(ctx, msg.channel_id).await;

    Ok(())
}

#[command]
async fn new(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(())
    }

    let channel_test = args.single::<String>().unwrap_or(msg.channel_id.mention());

    let channel_id = match parse_channel(&channel_test) {
        Some(channel_id) => ChannelId::from(channel_id),
        None => {
            msg.channel_id.say(ctx, RoyalError::MissingError("mentioned channel in position 1")).await?;

            return Ok(())
        }
    };

    let msg_id = match args.single::<u64>() {
        Ok(msg_id) => msg_id,
        Err(_) => {
            msg.channel_id.say(ctx, RoyalError::MissingError("message ID in position 2")).await?;

            return Ok(())
        }
    };

    if ctx.http.get_message(channel_id.0, msg_id).await.is_err() {
        msg.channel_id.say(ctx, RoyalError::MissingError("valid message ID in position 2")).await?;

        return Ok(())
    }

    let emoji_string = match args.single::<String>() {
        Ok(string) => string,
        Err(_) => {
            msg.channel_id.say(ctx, RoyalError::MissingError("emoji in position 3")).await?;

            return Ok(())
        }
    };

    let reaction_emoji = match check_emoji(&emoji_string) {
        Ok(emoji) => emoji,
        Err(e) => {
            msg.channel_id.say(ctx, e).await?;

            return Ok(())
        }
    };

    if msg.mention_roles.is_empty() {
        msg.channel_id.say(ctx, RoyalError::MissingError("role mention in position 4")).await?;

        return Ok(())
    }

    let storage = WizardIntermediate {
        message_id: msg_id,
        channel_id,
        role_id: msg.mention_roles[0],
        emoji: reaction_emoji
    };

    add_reaction(ctx, msg, storage).await?;

    Ok(())
}

#[command]
async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(())
    }

    let channel_test = args.single::<String>().unwrap_or(msg.channel_id.mention());

    let channel_id = match parse_channel(&channel_test) {
            Some(channel_id) => ChannelId::from(channel_id),
            None => {
                msg.channel_id.say(ctx, RoyalError::MissingError("mentioned channel in position 1")).await?;
        
                return Ok(())
            }
        };

    let msg_id = match args.single::<u64>() {
        Ok(msg_id) => msg_id,
        Err(_) => {
            msg.channel_id.say(ctx, RoyalError::MissingError("message ID in position 2")).await?;

            return Ok(())
        }
    };

    if ctx.http.get_message(channel_id.0, msg_id).await.is_err() {
        msg.channel_id.say(ctx, RoyalError::MissingError("valid message ID in position 2")).await?;

        return Ok(())
    }

    let emoji_string = match args.single::<String>() {
        Ok(string) => string,
        Err(_) => {
            msg.channel_id.say(ctx, RoyalError::MissingError("emoji in position 3")).await?;

            return Ok(())
        }
    };

    let reaction_emoji = match check_emoji(&emoji_string) {
        Ok(emoji) => emoji,
        Err(e) => {
            msg.channel_id.say(ctx, e).await?;

            return Ok(())
        }
    };

    let emoji = reaction_emoji.emoji.unwrap();

    let reaction_type =     
        if reaction_emoji.animated.is_some() && reaction_emoji.name.is_some() {
            ReactionType::Custom {
                animated: reaction_emoji.animated.unwrap(),
                id: EmojiId::from(emoji.parse::<u64>()?),
                name: Some(reaction_emoji.name.unwrap())
            }
        } else {
            ReactionType::Unicode(emoji.clone())
        };

    ctx.http.delete_reaction(channel_id.0, msg_id, None, &reaction_type).await?;

    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let check = sqlx::query!("SELECT EXISTS(SELECT 1 FROM reaction_roles WHERE message_id = $1 AND emoji = $2)", 
            msg_id as i64, emoji)
        .fetch_one(pool).await?;

    if check.exists.unwrap() {
        sqlx::query!("DELETE FROM reaction_roles WHERE message_id = $1 AND emoji = $2",
                msg_id as i64, emoji)
            .execute(pool).await?;

        msg.channel_id.say(ctx, "Reaction successfully removed from the database!").await?;
    } else {
        msg.channel_id.say(ctx, "Doesn't look like that role/emoji combo exists! Try a different message/channel Id?").await?;
    }

    Ok(())
}

#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let role_data = sqlx::query!("SELECT * FROM reaction_roles WHERE guild_id = $1", guild_id.0 as i64)
        .fetch_all(pool).await?;

    let mut msg_id_string = String::new();
    let mut emoji_string = String::new();
    let mut role_string = String::new();

    for i in role_data {
        let msg_url = get_message_url(
            guild_id, ChannelId::from(i.channel_id as u64), MessageId::from(i.message_id as u64));

        msg_id_string.push_str(&format!("[url]({}) \n", msg_url));

        role_string.push_str(&format!("{} \n", RoleId::from(i.role_id as u64).mention()));

        if i.animated.is_some() && i.emoji_name.is_some() {
            let emoji = get_custom_emoji(i.emoji, i.emoji_name.unwrap(), i.animated.unwrap());

            emoji_string.push_str(&format!("{} \n", emoji));
        } else {
            emoji_string.push_str(&format!("{} \n", i.emoji));
        }
    }

    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.color(0xfac916);
            e.title("Reaction Roles");
            e.description("If the message URL or role mentions are invalid, please use the remove command!");
            e.field("Messages", msg_id_string, true);
            e.field("Emojis", emoji_string, true);
            e.field("Roles", role_string, true);
            e
        })
    }).await?;

    Ok(())
}

#[command]
async fn wizard(ctx: &Context, msg: &Message) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(())
    }

    let sent_message = msg.channel_id.say(ctx, 
        concat!("Welcome to Reaction Role configuration! \n",
        "Please react ✅ to proceed and ❌ to abort!")).await?;

    sent_message.react(ctx, ReactionType::Unicode(String::from("✅"))).await?;
    sent_message.react(ctx, ReactionType::Unicode(String::from("❌"))).await?;

    let wrapped_action = sent_message.await_reaction(ctx)
        .timeout(Duration::from_secs(120)).await;

    match wrapped_action {
        Some(action) => {
            let reaction = action.as_inner_ref();

            if let ReactionType::Unicode(emoji) = &reaction.emoji {
                if emoji == "✅" {
                    let storage = WizardIntermediate::default();

                    get_message(ctx, msg, storage).await?;
                }
                else if emoji == "❌" {
                    msg.channel_id.say(ctx, "Aborting...").await?;
                }
            }       
        },
        None => {
            msg.channel_id.say(ctx, "Timed out").await?;
        }
    }

    Ok(())
}

async fn get_message(ctx: &Context, msg: &Message, mut storage: WizardIntermediate) -> CommandResult {
    msg.channel_id.say(ctx, "Alright! Please give a channel mention followed by a message id for me to work with!").await?;

    loop {
        let id_message = msg.author.await_reply(ctx)
            .timeout(Duration::from_secs(120)).await;

        match id_message {
            Some(msg) => {
                let mut args = Args::new(&msg.content, &[Delimiter::Single(' ')]);

                let channel_test = args.single::<String>().unwrap();

                storage.channel_id = match parse_channel(&channel_test) {
                    Some(channel_id) => ChannelId::from(channel_id),
                    None => {
                        msg.channel_id.say(ctx, RoyalError::MissingError("mentioned channel")).await?;

                        continue
                    }
                };

                storage.message_id = match args.single::<u64>() {
                    Ok(msg_id) => msg_id,
                    Err(_) => {
                        msg.channel_id.say(ctx, RoyalError::MissingError("message ID")).await?;

                        continue
                    }
                };
            
                if ctx.http.get_message(storage.channel_id.0, storage.message_id).await.is_err() {
                    msg.channel_id.say(ctx, RoyalError::MissingError("valid message ID")).await?;
            
                    continue
                }

                break
            },
            None => {
                msg.channel_id.say(ctx, "Timed out").await?;

                return Ok(())
            }
        }
    }

    get_emoji(ctx, msg, storage).await?;

    Ok(())
}

async fn get_emoji(ctx: &Context, msg: &Message, mut storage: WizardIntermediate) -> CommandResult {
    msg.channel_id.say(ctx, 
        concat!("Awesome! Now please give me the emoji you want to use. \n",
        "Note: The emoji has to be from a server the BOT is in! \n",
        "The best option would be to use your server's custom emojis or unicode!")).await?;

    loop {
        let emoji_message = msg.author.await_reply(ctx)
            .timeout(Duration::from_secs(120)).await;
        
        match emoji_message {
            Some(msg) => {
                let mut args = Args::new(&msg.content, &[Delimiter::Single(' ')]);
                let emoji_string = args.single::<String>().unwrap();

                storage.emoji = match check_emoji(&emoji_string) {
                    Ok(emoji) => emoji,
                    Err(e) => {
                        msg.channel_id.say(ctx, e).await?;

                        continue
                    }
                };

                break
            },
            None => {
                msg.channel_id.say(ctx, "Timed out").await?;

                return Ok(())
            }
        }
    }

    get_role(ctx, msg, storage).await?;

    Ok(())
}

async fn get_role(ctx: &Context, msg: &Message, mut storage: WizardIntermediate) -> CommandResult {
    msg.channel_id.say(ctx, "Sounds good! Now, please give me a role mention that you want to assign!").await?;

    loop {
        let role_message = msg.author.await_reply(ctx)
            .timeout(Duration::from_secs(120)).await;
        
        match role_message {
            Some(msg) => {
                if msg.mention_roles.is_empty() {
                    msg.channel_id.say(ctx, RoyalError::MissingError("role mention")).await?;

                    continue
                }

                storage.role_id = msg.mention_roles[0];

                break
            },
            None => {
                msg.channel_id.say(ctx, "Timed out").await?;

                return Ok(())
            }
        }
    }

    add_reaction(ctx, msg, storage).await?;

    Ok(())
}

async fn add_reaction(ctx: &Context, msg: &Message, storage: WizardIntermediate) -> CommandResult {
    let channel_id = storage.channel_id;
    let msg_id = storage.message_id;
    let role_id = storage.role_id;
    let guild_id = msg.guild_id.unwrap();
    let reaction_emoji = storage.emoji;
    let emoji = reaction_emoji.emoji.unwrap();

    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionPool>().unwrap();

    let check = sqlx::query!("SELECT EXISTS(SELECT 1 FROM reaction_roles WHERE message_id = $1 AND (role_id = $2 OR emoji = $3))",
            msg_id as i64, role_id.0 as i64, emoji.to_string())
        .fetch_one(pool).await?;

    if check.exists.unwrap() {
        msg.channel_id.say(ctx, "Looks like this role/emoji combo already exists. Aborting...").await?;

        return Ok(())
    }

    let reaction_type =     
        if reaction_emoji.animated.is_some() && reaction_emoji.name.is_some() {
            ReactionType::Custom {
                animated: reaction_emoji.animated.unwrap(),
                id: EmojiId::from(emoji.parse::<u64>()?),
                name: Some(reaction_emoji.name.clone().unwrap())
            }
        } else {
            ReactionType::Unicode(emoji.clone())
        };

    match ctx.http.create_reaction(channel_id.0, msg_id, &reaction_type).await {
        Ok(_) => {
            msg.channel_id.say(ctx, "Reaction successfully added! Check the given message!").await?;

            sqlx::query!("INSERT INTO reaction_roles VALUES($1, $2, $3, $4, $5, $6, $7)",
                    msg_id as i64, guild_id.0 as i64, channel_id.0 as i64, emoji, role_id.0 as i64, reaction_emoji.animated, reaction_emoji.name)
                .execute(pool).await?;
        },
        Err(_) => {
            msg.channel_id.say(ctx, 
                concat!("Reaction unsuccessful. Please make sure the bot has the `Use External Emojis` and `Add Reactions` permissions!",
                "\nTo use this emoji, the bot has to be in the original server!")).await?;
        }
    }

    Ok(())
}

fn check_emoji(test_string: &str) -> CommandResult<ReactionEmoji> {
    let mut emoji_struct = ReactionEmoji::default();

    if let Some(custom) = parse_emoji(test_string) {
        emoji_struct.emoji = Some(custom.id.to_string());
        emoji_struct.animated = Some(custom.animated);
        emoji_struct.name = Some(custom.name);
    } else {
        if let Ok(emoji_char) =  test_string.parse::<char>() {
            if is_emoji(emoji_char) {
                emoji_struct.emoji = Some(test_string.to_owned());
            }
        } else {
            return Err("Please provide a emoji ID in position 3!".into())
        }
    }

    Ok(emoji_struct)
}

pub async fn reaction_role_help(ctx: &Context, channel_id: ChannelId) {
    let content = concat!(
        "wizard: Best for first-time users! A simple wizard to get a reaction role set up \n\n",
        "new (channel mention) <message ID> <emoji> <role mention>: For experienced users! Creates a reaction role in one command. \n", 
            "Channel ID defaults to the current channel. \n\n",
        "remove (channel mention) <message ID> <emoji>: Removes a reaction role on a given message. \n",
            "Channel ID defaults to the current channel.");
    
    
    let _ = channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Reaction Role help");
            e.description(concat!("Description: Creates/removes reaction roles \n The main command is `rr`. \n", 
                "Custom emojis must come from servers that the BOT is in!"));
            e.field("Subcommands", content, false);
            e
        })
    }).await;
}
