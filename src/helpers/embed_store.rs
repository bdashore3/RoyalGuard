use serenity::{
    builder::CreateEmbed,
    model::misc::Mentionable,
    model::{
        id::{ChannelId, RoleId},
        prelude::User,
    },
};

pub fn get_ban_embed(user: &User, reason: &str, use_id: bool) -> CreateEmbed {
    let mut eb = CreateEmbed::default();

    eb.color(0xff0000);

    if use_id {
        eb.title("New Ban by ID");
    } else {
        eb.title("New Ban");
    }

    eb.thumbnail(match user.avatar_url() {
        Some(avatar_url) => avatar_url,
        None => user.default_avatar_url(),
    });

    eb.field("Username", user.mention(), false);
    eb.field("Reason", reason, false);

    eb
}

pub fn get_unban_embed(user: &User, use_id: bool) -> CreateEmbed {
    let mut eb = CreateEmbed::default();

    eb.color(0x32cd32);

    if use_id {
        eb.title("New Unban by ID");
    } else {
        eb.title("New Unban");
    }

    eb.thumbnail(match user.avatar_url() {
        Some(avatar_url) => avatar_url,
        None => user.default_avatar_url(),
    });

    eb.field("Username", user.mention(), false);

    eb
}

pub fn get_kick_embed(user: &User, reason: &str, use_id: bool) -> CreateEmbed {
    let mut eb = CreateEmbed::default();

    eb.color(0xff0000);

    if use_id {
        eb.title("New Kick by ID");
    } else {
        eb.title("New Kick");
    }

    eb.thumbnail(match user.avatar_url() {
        Some(avatar_url) => avatar_url,
        None => user.default_avatar_url(),
    });

    eb.field("Username", user.mention(), false);
    eb.field("Reason", reason, false);

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
        None => user.default_avatar_url(),
    });

    eb.field("Username", user.mention(), false);
    eb.field("Warn Amount", warn_number, false);

    eb
}

pub fn get_mute_embed(
    user: &User,
    new_mute: bool,
    use_time: bool,
    mute_time_length: Option<&str>,
) -> CreateEmbed {
    let mut eb = CreateEmbed::default();

    eb.thumbnail(match user.avatar_url() {
        Some(avatar_url) => avatar_url,
        None => user.default_avatar_url(),
    });

    eb.field("Username", user.mention(), false);

    if new_mute {
        eb.color(0xcd5c5c);
        eb.title("New Mute");

        if use_time {
            eb.description("This mute will expire after the given time!");
            eb.field("Time Length", mute_time_length.unwrap(), false);
        } else {
            eb.description("This mute has to be removed by an admin!");
        }
    } else {
        eb.color(0x32cd32);
        eb.title("Removed Mute");
    }

    eb
}

pub fn get_channel_embed(channel_id: ChannelId, channel_type: &str) -> CreateEmbed {
    let mut eb = CreateEmbed::default();

    eb.color(0xa5f2f3);
    eb.title(format!("New {} Channel", channel_type));
    eb.description(format!("New channel: {}", channel_id.mention()));
    eb
}

pub fn get_new_member_embed(
    message: String,
    channel_id: ChannelId,
    message_type: &str,
) -> CreateEmbed {
    let mut eb = CreateEmbed::default();

    eb.color(0x30d5c8);
    eb.title(format!("{} information", message_type));
    eb.description(format!(
        "Current welcome/leave channel: {}",
        channel_id.mention()
    ));
    eb.field("Message", format!("```{} \n```", message), false);
    eb
}

pub fn get_welcome_roles_embed(role_ids: Vec<RoleId>) -> CreateEmbed {
    let mut eb = CreateEmbed::default();

    eb.title("Current Welcome Roles");
    eb.color(0x30d5c8);

    eb.description("These roles are assigned when a new member joins the server.");

    let mut role_string = String::new();

    for role_id in role_ids {
        role_string.push_str(&format!("{} ", role_id.mention()));
    }

    eb.field("Roles", role_string, false);

    eb
}
