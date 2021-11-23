use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    helpers::delete_buffer, helpers::mute_helper::load_mute_timers, reactions::reaction_roles,
    structures::cmd_data::BotId, ConnectionPool,
};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        channel::{GuildChannel, Message, Reaction},
        event::MessageUpdateEvent,
        guild::{Guild, GuildUnavailable, Member},
        id::{ChannelId, GuildId, MessageId, RoleId},
        prelude::{Activity, Mentionable, Ready, User},
    },
};

// Event handler for when the bot starts
pub struct SerenityHandler {
    pub run_loop: AtomicBool,
}

#[async_trait]
impl EventHandler for SerenityHandler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        if self.run_loop.load(Ordering::Relaxed) {
            self.run_loop.store(false, Ordering::Relaxed);

            println!("Running guild pruner!");
            if let Err(e) = delete_buffer::guild_pruner(&ctx).await {
                panic!("Error when pruning guilds! {}", e);
            }

            println!("Loading mute timers!");
            if let Err(e) = load_mute_timers(&ctx).await {
                panic!("Error when restoring mutes! {}", e);
            }

            println!("Starting guild deletion loop!");

            let ctx_clone = ctx.clone();
            tokio::spawn(async move {
                if let Err(e) = delete_buffer::guild_removal_loop(ctx_clone).await {
                    panic!("Delete buffer failed to start!: {}", e);
                };
            });

            println!("Setting activity...");
            ctx.shard
                .set_activity(Some(Activity::watching("the castle")));
        }
    }

    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, mut new_member: Member) {
        if new_member.user.bot {
            return;
        }

        let pool = ctx
            .data
            .read()
            .await
            .get::<ConnectionPool>()
            .cloned()
            .unwrap();

        let welcome_data = match sqlx::query!(
            "SELECT welcome_message, channel_id FROM new_members WHERE guild_id = $1",
            guild_id.0 as i64
        )
        .fetch_optional(&pool)
        .await
        {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error in fetching the welcome data: {}", e);
                return;
            }
        };

        if let Some(welcome_data) = welcome_data {
            if let Some(message) = welcome_data.welcome_message {
                let channel_id = ChannelId::from(welcome_data.channel_id as u64);

                let welcome_message = message
                    .replace("{user}", &new_member.user.mention().to_string())
                    .replace("{server}", &guild_id.name(&ctx).await.unwrap());

                let _ = channel_id.say(&ctx, welcome_message).await;
            }
        }

        let role_check = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM welcome_roles WHERE guild_id = $1)",
            guild_id.0 as i64
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        if role_check.exists.unwrap() {
            let welcome_roles = sqlx::query!(
                "SELECT role_id FROM welcome_roles WHERE guild_id = $1",
                guild_id.0 as i64
            )
            .fetch_all(&pool)
            .await
            .unwrap();

            for i in welcome_roles {
                if new_member
                    .add_role(&ctx, RoleId::from(i.role_id as u64))
                    .await
                    .is_err()
                {
                    sqlx::query!(
                        "DELETE FROM welcome_roles WHERE guild_id = $1 AND role_id = $2",
                        guild_id.0 as i64,
                        i.role_id
                    )
                    .execute(&pool)
                    .await
                    .unwrap();
                }
            }
        }
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        guild_id: GuildId,
        user: User,
        _member_data_if_available: Option<Member>,
    ) {
        let pool = ctx
            .data
            .read()
            .await
            .get::<ConnectionPool>()
            .cloned()
            .unwrap();

        if user.bot {
            return;
        }

        let leave_data = match sqlx::query!(
            "SELECT leave_message, channel_id FROM new_members WHERE guild_id = $1",
            guild_id.0 as i64
        )
        .fetch_optional(&pool)
        .await
        {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error in fetching the welcome data: {}", e);
                return;
            }
        };

        if let Some(leave_data) = leave_data {
            if let Some(message) = leave_data.leave_message {
                let channel_id = ChannelId::from(leave_data.channel_id as u64);

                let leave_message = message
                    .replace(
                        "{user}",
                        &format!("**{}#{}**", user.name, user.discriminator),
                    )
                    .replace("{server}", &guild_id.name(&ctx).await.unwrap());

                let _ = channel_id.say(&ctx, leave_message).await;
            }
        }
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        let pool = ctx
            .data
            .read()
            .await
            .get::<ConnectionPool>()
            .cloned()
            .unwrap();

        if let Err(e) = delete_buffer::add_new_guild(&pool, guild.id, is_new).await {
            eprintln!("Error in guild creation! (ID {}): {}", guild.id.0, e);
        }
    }

    async fn guild_delete(&self, ctx: Context, incomplete: GuildUnavailable, _full: Option<Guild>) {
        let pool = ctx
            .data
            .read()
            .await
            .get::<ConnectionPool>()
            .cloned()
            .unwrap();

        if let Err(e) = delete_buffer::mark_for_deletion(&pool, incomplete.id).await {
            eprintln!(
                "Error in marking for deletion! (ID {}): {}",
                incomplete.id.0, e
            );
        }
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        if let Err(e) = reaction_roles::dispatch_event(&ctx, &add_reaction, false).await {
            eprintln!(
                "Error in reaction dispatch! (ID {}): {}",
                add_reaction.guild_id.unwrap().0,
                e
            );

            let _ = add_reaction.channel_id.say(ctx, concat!("Looks like there was an error when you reacted! \n",
                "Please make sure you have the `Add Reactions` permission enabled for both the channel and the bot role!")).await;
        }
    }

    async fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        if let Err(e) = reaction_roles::dispatch_event(&ctx, &removed_reaction, true).await {
            eprintln!(
                "Error in reaction dispatch! (ID {}): {}",
                removed_reaction.guild_id.unwrap().0,
                e
            );

            let _ = removed_reaction.channel_id.say(ctx, concat!("Looks like there was an error when you reacted! \n",
                "Please make sure you have the `Add Reactions` permission enabled for both the channel and the bot role!")).await;
        }
    }

    async fn reaction_remove_all(
        &self,
        ctx: Context,
        _channel_id: ChannelId,
        message_id: MessageId,
    ) {
        let pool = ctx
            .data
            .read()
            .await
            .get::<ConnectionPool>()
            .cloned()
            .unwrap();

        if let Err(e) = delete_buffer::delete_leftover_reactions(&pool, message_id).await {
            println!("Error when deleting reactions in message delete! {}", e);
        }
    }

    async fn message_update(
        &self,
        ctx: Context,
        optional_old_msg: Option<Message>,
        optional_new_msg: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        let guild_id = match event.guild_id {
            Some(guild_id) => guild_id,
            None => return,
        };

        let (pool, bot_id) = {
            let data = ctx.data.read().await;
            let pool = data.get::<ConnectionPool>().cloned().unwrap();
            let bot_id = data.get::<BotId>().cloned().unwrap();

            (pool, bot_id)
        };

        let wrapped_request = sqlx::query!(
            "SELECT guild_id, message_channel_id FROM logging WHERE guild_id = $1",
            guild_id.0 as i64
        )
        .fetch_optional(&pool)
        .await
        .unwrap();

        if let Some(request) = wrapped_request {
            let username = match &event.author {
                Some(author) => {
                    if author.id == bot_id {
                        return;
                    }

                    let temp_username = format!("**{}#{}**", author.name, author.discriminator);
                    temp_username
                }
                None => "No author provided".to_string(),
            };

            let message_log_channel_id =
                ChannelId::from(request.message_channel_id.unwrap() as u64);

            let old_msg_content = match &optional_old_msg {
                Some(old_msg) => &old_msg.content,
                None => "No old message was provided",
            };

            let new_msg_content = match &optional_new_msg {
                Some(new_msg) => &new_msg.content,
                None => event
                    .content
                    .as_deref()
                    .unwrap_or("No new message was provided"),
            };

            let edit_embed_string = format!(
                "{} edited message (with ID: {}) \nin channel {} (with ID: {})",
                username,
                event.id,
                event.channel_id.mention(),
                event.channel_id
            );
            let content_string = format!("Old: {}\n New: {}", old_msg_content, new_msg_content);

            let _ = message_log_channel_id
                .send_message(ctx, |m| {
                    m.content(edit_embed_string);
                    m.embed(|e| {
                        e.color(0xffc0cb);
                        e.description(content_string);
                        e
                    })
                })
                .await;
        }
    }

    async fn message_delete(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        message_id: MessageId,
        guild_id: Option<GuildId>,
    ) {
        let (pool, bot_id) = {
            let data = ctx.data.read().await;
            let pool = data.get::<ConnectionPool>().cloned().unwrap();
            let bot_id = data.get::<BotId>().cloned().unwrap();

            (pool, bot_id)
        };

        if let Err(e) = delete_buffer::delete_leftover_reactions(&pool, message_id).await {
            println!("Error when deleting reactions in message delete! {}", e);
        }

        let guild_id = match guild_id {
            Some(guild_id) => guild_id,
            None => return,
        };

        let wrapped_request = sqlx::query!(
            "SELECT guild_id, message_channel_id FROM logging WHERE guild_id = $1",
            guild_id.0 as i64
        )
        .fetch_optional(&pool)
        .await
        .unwrap();

        if let Some(request) = wrapped_request {
            let message_log_channel_id =
                ChannelId::from(request.message_channel_id.unwrap() as u64);

            let message = match ctx.cache.message(channel_id, message_id).await {
                Some(message) => message,
                None => match ctx.http.get_message(channel_id.0, message_id.0).await {
                    Ok(message) => message,
                    Err(_) => return,
                },
            };

            if message.author.id == bot_id {
                return;
            }

            let username = format!(
                "**{}#{}**",
                message.author.name, message.author.discriminator
            );

            let delete_msg_description = format!(
                "{} deleted message (with ID: {}) \nin channel {} (with ID: {})",
                username,
                message.id,
                channel_id.mention(),
                channel_id
            );

            let _ = message_log_channel_id
                .send_message(ctx, |m| {
                    m.content(delete_msg_description);
                    m.embed(|e| {
                        e.color(0xff4c4c);
                        e.description(message.content);
                        e
                    })
                })
                .await;
        }
    }

    async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
        if let Err(e) = thread.id.join_thread(ctx).await {
            println!("Error in thread join! (ID {}): {}", thread.id, e);
        }
    }
}
