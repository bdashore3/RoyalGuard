using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using RoyalGuard.Helpers.Commands;

namespace RoyalGuard.Modules
{
    public class Bans
    {
        private readonly StringRenderer _stringRenderer;

        public Bans(StringRenderer stringRenderer)
        {
            _stringRenderer = stringRenderer;
        }
        public async Task BanUser(DiscordMessage message)
        {
            string reason = _stringRenderer.RemoveExtras(message, 2);
            string username = message.MentionedUsers[0].Username;
            await message.Channel.Guild.BanMemberAsync(message.MentionedUsers[0].Id, 0, reason);
            await message.RespondAsync($"Banned user `{username}` for reason: {reason}");
        }
    }
}