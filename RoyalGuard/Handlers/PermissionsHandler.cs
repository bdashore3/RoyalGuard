using System;
using System.Threading.Tasks;
using DSharpPlus;
using DSharpPlus.Entities;

namespace RoyalGuard.Handlers
{
    public class PermissionsHandler
    {
        public bool CheckAdmin(DiscordMessage message)
        {
            DiscordMember member = message.Author as DiscordMember;
            if (member.PermissionsIn(message.Channel).HasPermission(Permissions.Administrator))
                return true;
            message.RespondAsync("You can't execute this command!");
            return false;
        } 

        public bool CheckBanPermission(DiscordMessage message)
        {
            DiscordMember member = message.Author as DiscordMember;
            if (member.PermissionsIn(message.Channel).HasPermission(Permissions.BanMembers))
                return true;
            return false;
        }

        public bool CheckAdminFromMention(DiscordUser user, DiscordChannel channel)
        {
            DiscordMember member = user as DiscordMember;
            if (member.PermissionsIn(channel).HasPermission(Permissions.Administrator))
                return true;
            return false;
        }
    }
}
