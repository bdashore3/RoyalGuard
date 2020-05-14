using System;
using System.Threading.Tasks;
using DSharpPlus;
using DSharpPlus.Entities;

namespace RoyalGuard.Handlers
{
    public class PermissionsHandler
    {
        public bool CheckPermission(DiscordMessage message, Permissions permission)
        {
            DiscordMember member = message.Author as DiscordMember;
            if (member.PermissionsIn(message.Channel).HasPermission(permission))
                return true;
            
            if (permission == Permissions.Administrator)
                message.RespondAsync("You can't execute this command because you do not have the administrator permission in the server!");
            else
                message.RespondAsync("You cannot execute this command since you are not a moderator in this server!");
            
            return false;
        }

        public bool CheckMentionedPermission(DiscordUser user, DiscordChannel channel, Permissions permission)
        {
            DiscordMember member = user as DiscordMember;
            if (member.PermissionsIn(channel).HasPermission(permission))
                return true;
            
            return false;
        }
    }
}
