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
            message.RespondAsync("You can't execute this command because you do not have the administrator permission in the server!");
            return false;
        } 

        // Checks if the message author can ban users (upcoming moderator permission)
        public bool CheckMod(DiscordMessage message)
        {
            DiscordMember member = message.Author as DiscordMember;
            if (member.PermissionsIn(message.Channel).HasPermission(Permissions.BanMembers))
                return true;
            message.RespondAsync("You cannot execute this command since you are not a moderator in this server! (Has ban permissions)");
            return false;
        }

        // Checks the mentioned user for admin permissions
        public bool CheckAdminFromMention(DiscordUser user, DiscordChannel channel)
        {
            DiscordMember member = user as DiscordMember;
            if (member.PermissionsIn(channel).HasPermission(Permissions.Administrator) || member.PermissionsIn(channel).HasPermission(Permissions.BanMembers))
                return true;
            return false;
        }
    }
}
