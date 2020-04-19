using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Modules;
using RoyalGuard.Handlers;

namespace RoyalGuard.Commands
{
    public class CommandHandler
    {
        private readonly StringRenderer _stringRenderer;
        private readonly Bans _bans;
        private readonly Mutes _mutes;
        private readonly Warns _warns;
        private readonly PermissionsHandler _permissions;
        private readonly NewMemberHandler _newMemberHandler;
        private readonly Help _help;
        private readonly PrefixHelper _prefixHelper;
        public CommandHandler(
            StringRenderer stringRenderer, 
            Bans bans, Mutes mutes, 
            Warns warns, PermissionsHandler permissions, 
            NewMemberHandler newMemberHandler, 
            PrefixHelper prefixHelper,
            Help help)
        {
            _stringRenderer = stringRenderer;
            _bans = bans;
            _mutes = mutes;
            _warns = warns;
            _permissions = permissions;
            _newMemberHandler = newMemberHandler;
            _prefixHelper = prefixHelper;
            _help = help;
        }

        public async Task HandleCommand(DiscordMessage message)
        {
            switch (_stringRenderer.GetCommand(message))
            {
                case "ping":
                    await Other.Ping(message);
                    break;

                case "ban":
                    if (!_permissions.CheckAdmin(message))
                        break;

                    await _bans.BanUser(message);
                    break;

                case "unban":
                    if (!_permissions.CheckAdmin(message))
                        break;

                    await _bans.UnbanUser(message, false);
                    break;

                case "unbanid":
                case "unbanbyid":
                case "unbanId":
                    if (!_permissions.CheckAdmin(message))
                        break;
                    
                    await _bans.UnbanUser(message, true);
                    break;

                case "warn":
                    if (!_permissions.CheckAdmin(message))
                        break;

                    await _warns.WarnUser(message);
                    break;

                case "unwarn":
                    if (!_permissions.CheckAdmin(message))
                        break;

                    await _warns.UnwarnUser(message);
                    break;

                case "getwarns":
                    await _warns.GetWarns(message);
                    break;

                case "repeat":
                    Console.WriteLine(message.Content);
                    break;
                
                case "welcome":
                    if (!_permissions.CheckAdmin(message))
                        break;

                    await _newMemberHandler.HandleConfiguration(message, "welcome");
                    break;
                
                case "leave":
                    if (!_permissions.CheckAdmin(message))
                        break;

                    await _newMemberHandler.HandleConfiguration(message, "leave");
                    break;

                case "help":
                    await _help.DirectHelp(message);
                    break;

                case "setprefix":
                case "prefix":
                    await _prefixHelper.HandleConfiguration(message);
                    break;
                
                case "getprefix":
                    await _prefixHelper.GetPrefix(message);
                    break;

                case "mute":
                    if (!_permissions.CheckAdmin(message))
                        break;

                    await _mutes.MuteUser(message);
                    break;

                case "unmute":
                    if (!_permissions.CheckAdmin(message))
                        break;

                    await _mutes.UnmuteUser(message);
                    break;
            }
        }
    }
}
