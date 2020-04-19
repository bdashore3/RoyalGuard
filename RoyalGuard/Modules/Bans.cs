using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Handlers;
using RoyalGuard.Helpers;

namespace RoyalGuard.Modules
{
    public class Bans
    {
        private readonly StringRenderer _stringRenderer;
        private readonly PermissionsHandler _permissions;

        public Bans(StringRenderer stringRenderer, PermissionsHandler permissions)
        {
            _stringRenderer = stringRenderer;
            _permissions = permissions;
        }
        public async Task BanUser(DiscordMessage message)
        {
            if (_permissions.CheckAdminFromMention(message.MentionedUsers[0], message.Channel))
            {
                await message.RespondAsync("I can't ban an administrator! Please demote the user then try again.");
                return;
            }

            string reason = _stringRenderer.RemoveExtras(message, 2);
            ulong userId = message.MentionedUsers[0].Id;
            string username = $"<@!{userId}>";
            await message.Channel.Guild.BanMemberAsync(userId, 0, reason);

            if (reason == null)
                reason = "No reason given.";

            DiscordEmbed banEmbed = EmbedStore.GetBanEmbed(message.MentionedUsers[0].AvatarUrl, username, reason);

            await message.RespondAsync("", false, banEmbed);
        }

        public async Task UnbanUser(DiscordMessage message, bool useId)
        {
            ulong userId;

            Console.WriteLine(_stringRenderer.GetWordFromIndex(message, 1));

            if (useId)
                userId = UInt64.Parse(_stringRenderer.GetWordFromIndex(message, 1));
            else
                userId = message.MentionedUsers[0].Id;

            string username = $"<@!{userId}>";
            await message.Channel.Guild.UnbanMemberAsync(userId);

            DiscordEmbed unbanEmbed = EmbedStore.GetUnbanEmbed(username, useId);
            await message.RespondAsync("", false, unbanEmbed);
        }

        public static async Task BanHelp(DiscordMessage message)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();
            eb.WithTitle("Ban Help");
            eb.WithDescription("Description: Commands for Banning/Unbanning in a server");
            eb.AddField("Commands", "ban <mention> <reason>: Bans a user with a reason \n\n" +
                                    "unban <mention or id>: Unbans the mentioned user or provided ID");

            await message.RespondAsync("", false, eb.Build());
        }
    }
}