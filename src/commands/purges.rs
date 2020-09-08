use serenity::{
    prelude::*,
    model::prelude::*,
    framework::standard::{
        CommandResult,
        macros::command, Args
    }
};
use crate::{RoyalError, helpers::permissions_helper};

#[command]
async fn purge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !permissions_helper::check_moderator(ctx, msg, None).await? {
        return Ok(())
    }

    if args.is_empty() {
        purge_help(ctx, msg.channel_id).await;

        return Ok(())
    }

    let num = match args.single::<u64>() {
        Ok(num) => num,
        Err(_) => {
            msg.channel_id.say(ctx, RoyalError::MissingError("message id or integer")).await?;

            return Ok(())
        }
    };

    let use_id = ctx.http.get_message(msg.channel_id.0, num).await.is_ok();

    #[allow(unused_assignments)]
    let mut messages: Vec<Message> = Vec::new();

    if use_id {
        let start_msg_id = MessageId::from(num);

        messages = msg.channel_id.messages(ctx, |m| m.after(start_msg_id)).await?;
    } else {
        if num > 100 {
            msg.channel_id.say(ctx, RoyalError::MissingError("value less than or equal to 100!")).await?;
            
            return Ok(())
        }

        messages = msg.channel_id.messages(ctx, |m,| m.limit(num + 1)).await?;
    }

    if messages.len() > 100 {
        msg.channel_id.say(ctx, RoyalError::MissingError("value less than or equal to 100!")).await?;
        
        return Ok(())
    }

    match msg.channel_id.delete_messages(ctx, messages.into_iter().map(|m| m.id)).await {
        Ok(_) => {
            msg.channel_id.say(ctx, "Purge complete.").await?;
        },
        Err(e) => {
            msg.channel_id.say(ctx, "Can't delete messages older than 2 weeks!").await?;

            eprintln!("Purge Error in guild {}: {}", msg.guild_id.unwrap().0, e);
        }
    };

    Ok(())
}

pub async fn purge_help(ctx: &Context, channel_id: ChannelId) {
    let content = concat!(
        "purge <amount to remove>: Removes a specified amount of messages before the command. \n\n",
        "purge <ID of message to start from>: Removes all messages between the ID and the command.");
    
    let _ = channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Purge help");
            e.description("Description: Commands for bulk removal of messages in a server");
            e.field("Commands", content, false);
            e
        })
    }).await;
}
