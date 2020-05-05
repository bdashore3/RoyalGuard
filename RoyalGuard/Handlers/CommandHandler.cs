using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Modules;
using RoyalGuard.Handlers;
using RoyalGuard.Helpers.Data;

namespace RoyalGuard.Commands
{
    public class CommandHandler
    {
        // Variables and constructor for DI
        private readonly StringRenderer _stringRenderer;
        private readonly Bans _bans;
        private readonly Mutes _mutes;
        private readonly Warns _warns;
        private readonly PermissionsHandler _permissions;
        private readonly NewMemberHandler _newMemberHandler;
        private readonly Purge _purge;
        private readonly PrefixHelper _prefixHelper;
        private readonly Help _help;
        private readonly GuildInfoHelper _guildInfoHelper;
        public CommandHandler(
            StringRenderer stringRenderer, 
            Bans bans, Mutes mutes, 
            Warns warns, PermissionsHandler permissions, 
            NewMemberHandler newMemberHandler, 
            Purge purge,
            PrefixHelper prefixHelper,
            Help help,
            GuildInfoHelper guildInfoHelper)
        {
            _stringRenderer = stringRenderer;
            _bans = bans;
            _mutes = mutes;
            _warns = warns;
            _permissions = permissions;
            _newMemberHandler = newMemberHandler;
            _purge = purge;
            _prefixHelper = prefixHelper;
            _help = help;
            _guildInfoHelper = guildInfoHelper;
        }

        /*
         * Flow:
         * 1. Use StringRenderer to get the required parts of the message
         * 2. Pass the command through a switch which sends the message to
         *    its respective class
         *
         * If the user isn't an admin for some commands, tell the user
         * that he/she cannot execute the command!
         */
        public async Task HandleCommand(DiscordMessage message)
        {
            switch (_stringRenderer.GetCommand(message, false))
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

                    await _bans.UnbanUser(message);
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
                    await message.RespondAsync(message.Content);
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
                    await _prefixHelper.HandleConfiguration(message, false);
                    break;

                case "mute":
                    if (!_permissions.CheckAdmin(message))
                        break;

                    await _mutes.MuteUser(message);
                    break;
                
                case "mutechannel":
                    if (!_permissions.CheckAdmin(message))
                        break;
                    
                    await _mutes.ChangeMuteChannel(message);
                    break;

                case "unmute":
                    if (!_permissions.CheckAdmin(message))
                        break;

                    await _mutes.UnmuteUser(message);
                    break;
                
                case "purge":
                    await _purge.PurgeMessages(message);
                    break;
            }
        }

        public async Task HandleEmergency(DiscordMessage message)
        {
            switch (_stringRenderer.GetCommand(message, true))
            {
                case "resetprefix":
                    await _prefixHelper.ResetPrefix(message);
                    break;

                case "prefix":
                    await _prefixHelper.HandleConfiguration(message, true);
                    break;
                
                case "help":
                    await _help.SendEmergencyHelp(message);
                    break;

                case "init":
                    await _guildInfoHelper.GuildSetup(message);
                    break;
            }
        }
    }
}
