using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.PermissionsCheck;

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
            string username = message.MentionedUsers[0].Username;
            await message.Channel.Guild.BanMemberAsync(message.MentionedUsers[0].Id, 0, reason);
            await message.RespondAsync($"Banned user `{username}` for reason: {reason}");
        }

        public async Task UnbanUser(DiscordMessage message)
        {
            string username = message.MentionedUsers[0].Username;
            await message.Channel.Guild.UnbanMemberAsync(message.MentionedUsers[0].Id);
            await message.RespondAsync($"Unbanned user `{username}`. The user can now rejoin.");
        }
    }
}