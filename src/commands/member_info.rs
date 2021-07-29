use serenity::{
    builder::CreateEmbed,
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
#[aliases("minfo", "memberinfo")]
async fn get_member_info(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let member = if args.is_empty() {
        msg.member(ctx).await?
    } else {
        let user_id = args.single::<UserId>()?;

        guild_id.member(ctx, user_id).await?
    };

    let user = member.user;
    let nick = member
        .nick
        .unwrap_or_else(|| "No nickname here!".to_owned());
    let join_date = member.joined_at.as_ref().map(|d| d.to_rfc2822());
    let is_bot = if user.bot { "Yes" } else { "No" };
    let creation_date = user.id.created_at().to_rfc2822();

    let role_string = member
        .roles
        .iter()
        .take(4)
        .map(|role_id| format!("{} \n", role_id.mention()))
        .collect::<String>();

    let mut info_embed = CreateEmbed::default();

    info_embed.color(0xeb7e10);
    info_embed.author(|a| {
        a.name(&user.name);
        a.icon_url(&user.face());
        a
    });
    info_embed.field(
        "Discriminator",
        format!("#{:0>4}", user.discriminator),
        true,
    );
    info_embed.field("User ID", user.id.0, true);
    info_embed.field("Nickname", nick, true);
    info_embed.field("Mention", user.mention(), true);
    info_embed.field("Bot?", is_bot, true);
    info_embed.field("Role Excerpt", role_string, false);
    info_embed.field("Account Creation Date", creation_date, false);

    if let Some(join_date) = join_date {
        info_embed.field("Server Join Date", join_date, false);
    }

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.0 = info_embed.0;
                e
            })
        })
        .await?;

    Ok(())
}
