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
        private readonly PermissionsHandler _permissionsHandler;
        private readonly NewMemberHandler _newMemberHandler;
        private readonly Purge _purge;
        private readonly PrefixHelper _prefixHelper;
        private readonly Help _help;
        private readonly GuildInfoHelper _guildInfoHelper;
        private readonly Other _other;
        public CommandHandler(
            StringRenderer stringRenderer, 
            Bans bans, Mutes mutes, 
            Warns warns, PermissionsHandler permissionsHandler, 
            NewMemberHandler newMemberHandler, 
            Purge purge,
            PrefixHelper prefixHelper,
            Help help,
            GuildInfoHelper guildInfoHelper,
            Other other)
        {
            _stringRenderer = stringRenderer;
            _bans = bans;
            _mutes = mutes;
            _warns = warns;
            _permissionsHandler = permissionsHandler;
            _newMemberHandler = newMemberHandler;
            _purge = purge;
            _prefixHelper = prefixHelper;
            _help = help;
            _guildInfoHelper = guildInfoHelper;
            _other = other;
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
                    await _other.Ping(message);
                    break;

                case "ban":
                    if (!_permissionsHandler.CheckMod(message))
                        break;

                    await _bans.BanUser(message);
                    break;
                
                case "kick":
                    if (!_permissionsHandler.CheckMod(message))
                        break;
                    
                    await _other.KickUser(message);
                    break;

                case "unban":
                    if (!_permissionsHandler.CheckMod(message))
                        break;

                    await _bans.UnbanUser(message);
                    break;

                case "warn":
                    if (!_permissionsHandler.CheckMod(message))
                        break;

                    await _warns.WarnUser(message);
                    break;

                case "unwarn":
                    if (!_permissionsHandler.CheckMod(message))
                        break;

                    await _warns.UnwarnUser(message);
                    break;

                case "getwarns":
                    await _warns.GetWarns(message);
                    break;
                
                case "welcome":
                    if (!_permissionsHandler.CheckAdmin(message))
                        break;

                    await _newMemberHandler.HandleConfiguration(message, "welcome");
                    break;
                
                case "leave":
                    if (!_permissionsHandler.CheckAdmin(message))
                        break;

                    await _newMemberHandler.HandleConfiguration(message, "leave");
                    break;

                case "help":
                    await _help.DirectHelp(message);
                    break;
                
                case "support":
                    await _help.SendSupportMessage(message);
                    break;

                case "setprefix":
                case "prefix":
                    await _prefixHelper.HandleConfiguration(message, false);
                    break;

                case "mute":
                    if (!_permissionsHandler.CheckMod(message))
                        break;

                    await _mutes.MuteUser(message);
                    break;
                
                case "mutechannel":
                    if (!_permissionsHandler.CheckMod(message))
                        break;
                    
                    await _mutes.ChangeMuteChannel(message);
                    break;

                case "unmute":
                    if (!_permissionsHandler.CheckMod(message))
                        break;

                    await _mutes.UnmuteUser(message);
                    break;
                
                case "purge":
                    if (!_permissionsHandler.CheckMod(message))
                        break;

                    await _purge.PurgeMessages(message);
                    break;
            }
        }

        // Handle all commands in case of a bot emergency
        public async Task HandleEmergency(DiscordMessage message)
        {
            switch (_stringRenderer.GetCommand(message, true))
            {
                case "resetprefix":
                    await message.RespondAsync($"<@!{message.Author.Id}>, You are running an emergency command!");
                    await _prefixHelper.ResetPrefix(message);
                    break;

                case "prefix":
                    await message.RespondAsync($"<@!{message.Author.Id}>, You are running an emergency command!");
                    await _prefixHelper.HandleConfiguration(message, true);
                    break;
                
                case "help":
                    await message.RespondAsync($"<@!{message.Author.Id}>, You are running an emergency command!");
                    await _help.SendEmergencyHelp(message);
                    break;

                case "init":
                    if (!_permissionsHandler.CheckAdmin(message))
                        return;

                    await message.RespondAsync($"<@!{message.Author.Id}>, You are running an emergency command!");
                    await _guildInfoHelper.GuildSetup(message);
                    break;
            }
        }
    }
}
