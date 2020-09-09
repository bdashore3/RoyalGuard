use serenity::{
    prelude::*,
    model::prelude::*,
    framework::standard::CommandResult,
};
use crate::ConnectionPool;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use sqlx::PgPool;
use tokio::time::delay_for;

pub async fn mark_for_deletion(pool: &PgPool, guild_id: GuildId) -> CommandResult {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards?")
        .as_secs();
    
    sqlx::query!("INSERT INTO delete_time_store VALUES($1, $2)", 
            guild_id.0 as i64, current_time as i64 + 604800)
        .execute(pool).await?;

    Ok(())
}

pub async fn add_new_guild(pool: &PgPool, guild_id: GuildId, is_new: bool) -> CommandResult {
    let delete_data = sqlx::query!("SELECT EXISTS(SELECT 1 FROM delete_time_store WHERE guild_id = $1)", 
            guild_id.0 as i64)
        .fetch_one(pool).await?;

    if delete_data.exists.unwrap() {
        sqlx::query!("DELETE FROM delete_time_store WHERE guild_id = $1", guild_id.0 as i64)
            .execute(pool).await?;
    } else {
        if is_new {
            sqlx::query!("INSERT INTO guild_info VALUES($1, null, null, null) ON CONFLICT DO NOTHING",
                    guild_id.0 as i64)
                .execute(pool).await?;
        }
    }

    Ok(())
}

pub async fn guild_removal_loop(ctx: Context) -> CommandResult {
    loop {
        let pool = ctx.data.read().await
            .get::<ConnectionPool>().cloned().unwrap();
    
        let delete_data = sqlx::query!("SELECT guild_id, delete_time FROM delete_time_store")
            .fetch_all(&pool).await?;
        
        for i in delete_data {
            println!("Checking delete status on guild {}", i.guild_id);

            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards?")
                .as_secs() as i64;
            
            if i.delete_time <= current_time {
                println!("Deleting guild {} from the database \n", i.guild_id);
                sqlx::query!("DELETE FROM guild_info WHERE guild_id = $1", i.guild_id)
                    .execute(&pool).await?;
            } else {
                println!("Entry's time isn't greater than a week! Not deleting guild {}! \n", i.guild_id);
            }
        }
    
        delay_for(Duration::from_secs(345600)).await;
    }
}
