use serenity::{
    model::{prelude::User, id::UserId}, 
    client::Context
};

pub async fn get_user(ctx: &Context, user_id: &UserId) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
    let user = match ctx.cache.user(user_id).await {
        Some(user) => user,
        None => {
            ctx.http.get_user(user_id.0).await?
        }
    };

    Ok(user)
}