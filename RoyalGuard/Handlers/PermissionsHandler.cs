using System;
using System.Threading.Tasks;
using DSharpPlus;
using DSharpPlus.Entities;

namespace RoyalGuard.Handlers
{
    public class PermissionsHandler
    {
        // Checks if the message author is an admin
        public bool CheckAdmin(DiscordMessage message)
        {
            DiscordMember member = message.Author as DiscordMember;
            if (member.PermissionsIn(message.Channel).HasPermission(Permissions.Administrator))
                return true;
            message.RespondAsync("You can't execute this command!");
            return false;
        } 

        // Checks if the message author can ban users (upcoming moderator permission)
        public bool CheckBanPermission(DiscordMessage message)
        {
            DiscordMember member = message.Author as DiscordMember;
            if (member.PermissionsIn(message.Channel).HasPermission(Permissions.BanMembers))
                return true;
            return false;
        }

        // Checks the mentioned user for admin permissions
        public bool CheckAdminFromMention(DiscordUser user, DiscordChannel channel)
        {
            DiscordMember member = user as DiscordMember;
            if (member.PermissionsIn(channel).HasPermission(Permissions.Administrator))
                return true;
            return false;
        }
    }
}
