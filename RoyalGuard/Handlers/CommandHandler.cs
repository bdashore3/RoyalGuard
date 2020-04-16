using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Modules;
using RoyalGuard.PermissionsCheck;

namespace RoyalGuard.Commands
{
    public class CommandHandler
    {
        private readonly StringRenderer _stringRenderer;
        private readonly Bans _bans;
        private readonly Mutes _mutes;
        private readonly Warns _warns;
        private readonly PermissionsHandler _permissions;
        public CommandHandler(StringRenderer stringRenderer, Bans bans, Mutes mutes, Warns warns, PermissionsHandler permissions)
        {
            _stringRenderer = stringRenderer;
            _bans = bans;
            _mutes = mutes;
            _warns = warns;
            _permissions = permissions;
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
            }
        }
    }
}
