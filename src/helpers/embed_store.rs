use serenity::{
    model::prelude::User,
    builder::CreateEmbed,
    model::misc::Mentionable
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

    eb.field("Username", user.mention(), false);
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

    eb.field("Username", user.mention(), false);

    eb
}

pub fn get_warn_embed(user: &User, warn_number: i32, new_warn: bool) -> CreateEmbed {
    let mut eb = CreateEmbed::default();

    if new_warn {
        eb.color(0xcd5c5c);
        eb.title("New Warn");
    } else {
        eb.color(0x32cd32);
        eb.title("Removed Warn");
    }

    eb.thumbnail(match user.avatar_url() {
        Some(avatar_url) => avatar_url,
        None =>  user.default_avatar_url()
    });

    eb.field("Username", user.mention(), false);
    eb.field("Warn Amount", warn_number, false);

    eb
}