using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using RoyalGuard.Handlers;
using RoyalGuard.Helpers.Commands;

namespace RoyalGuard.Modules
{
    public class Other
    {
        private readonly PermissionsHandler _permissionsHandler;
        private readonly StringRenderer _stringRenderer;
        public Other(PermissionsHandler permissionsHandler, StringRenderer stringRenderer)
        {
            _permissionsHandler = permissionsHandler;
            _stringRenderer = stringRenderer;
        }

        public async Task Ping(DiscordMessage message)
        {
            await message.RespondAsync("Pong!");
        }

        public async Task KickUser(DiscordMessage message)
        {
            if (_stringRenderer.GetMessageCount(message) < 2)
            {
                await message.RespondAsync("Please mention the user you want to kick!");
                return;
            }

            if (_permissionsHandler.CheckAdminFromMention(message.MentionedUsers[0], message.Channel))
            {
                await message.RespondAsync("I can't kick an administrator/moderator! Please demote the user then try again.");
                return;
            }

            DiscordMember userToKick = message.MentionedUsers[0] as DiscordMember;

            await userToKick.RemoveAsync(null);
            await message.RespondAsync("", false, Helpers.EmbedStore.KickEmbed(userToKick.Mention));
        }
    }
}
