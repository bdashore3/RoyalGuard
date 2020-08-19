use serenity::{
    model::prelude::User,
    builder::CreateEmbed
};

pub fn get_ban_embed(use_id: bool, user: &User, reason: &str) -> CreateEmbed {
    let mut eb = CreateEmbed::default();

    eb.color(0xff0000);

    if use_id {
        eb.title("New Ban by ID");
    } else {
        eb.title("New Ban");
    }

    eb.thumbnail(match user.avatar_url() {
        Some(avatar_url) => avatar_url,
        None =>  user.default_avatar_url()
    });

    eb.field("Username", &user.name, false);
    eb.field("Reason", reason, false);

    eb
}

pub fn get_unban_embed(use_id: bool, user: &User) -> CreateEmbed {
    let mut eb = CreateEmbed::default();

    eb.color(0x32cd32);

    if use_id {
        eb.title("New Unban by ID");
    } else {
        eb.title("New Unban");
    }

    eb.thumbnail(match user.avatar_url() {
        Some(avatar_url) => avatar_url,
        None =>  user.default_avatar_url()
    });

    eb.description(format!("Username: {}", &user.name));

    eb
}